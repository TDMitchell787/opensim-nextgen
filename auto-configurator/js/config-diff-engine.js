// OpenSim Next Auto-Configurator - Configuration Diff Engine
// Intelligent diff generation and visualization for configuration changes

class ConfigDiffEngine {
    constructor() {
        this.diffOptions = {
            precision: 0.0001, // For floating point comparisons
            ignoreCase: false,
            ignoreWhitespace: false,
            ignoreComments: false
        };
    }

    generateDiff(originalConfig, currentConfig) {
        if (!originalConfig || !currentConfig) {
            throw new Error('Both original and current configurations are required');
        }

        const diff = {};
        const allKeys = new Set([
            ...Object.keys(originalConfig),
            ...Object.keys(currentConfig)
        ]);

        for (const key of allKeys) {
            const originalValue = originalConfig[key];
            const currentValue = currentConfig[key];
            
            if (this.isEqual(originalValue, currentValue)) {
                diff[key] = this.createUnchangedDiff(currentValue);
            } else {
                diff[key] = this.createSectionDiff(originalValue, currentValue, key);
            }
        }

        return diff;
    }

    createSectionDiff(original, current, sectionName) {
        if (original === undefined) {
            return this.createAddedDiff(current);
        }
        
        if (current === undefined) {
            return this.createDeletedDiff(original);
        }

        if (typeof original !== typeof current) {
            return this.createReplacedDiff(original, current);
        }

        if (this.isPrimitive(original)) {
            return this.createModifiedDiff(original, current);
        }

        if (Array.isArray(original)) {
            return this.createArrayDiff(original, current);
        }

        if (typeof original === 'object') {
            return this.createObjectDiff(original, current);
        }

        return this.createModifiedDiff(original, current);
    }

    createObjectDiff(originalObj, currentObj) {
        const diff = {};
        const allKeys = new Set([
            ...Object.keys(originalObj || {}),
            ...Object.keys(currentObj || {})
        ]);

        for (const key of allKeys) {
            const originalValue = originalObj?.[key];
            const currentValue = currentObj?.[key];

            if (originalValue === undefined) {
                diff[key] = this.createAddedDiff(currentValue);
            } else if (currentValue === undefined) {
                diff[key] = this.createDeletedDiff(originalValue);
            } else if (this.isEqual(originalValue, currentValue)) {
                diff[key] = this.createUnchangedDiff(currentValue);
            } else {
                diff[key] = this.createSectionDiff(originalValue, currentValue, key);
            }
        }

        return diff;
    }

    createArrayDiff(originalArray, currentArray) {
        const diff = [];
        const maxLength = Math.max(originalArray.length, currentArray.length);

        for (let i = 0; i < maxLength; i++) {
            const originalItem = originalArray[i];
            const currentItem = currentArray[i];

            if (i >= originalArray.length) {
                diff[i] = this.createAddedDiff(currentItem);
            } else if (i >= currentArray.length) {
                diff[i] = this.createDeletedDiff(originalItem);
            } else if (this.isEqual(originalItem, currentItem)) {
                diff[i] = this.createUnchangedDiff(currentItem);
            } else {
                diff[i] = this.createSectionDiff(originalItem, currentItem, `[${i}]`);
            }
        }

        // Detect moved items in arrays
        this.detectArrayMoves(originalArray, currentArray, diff);

        return diff;
    }

    detectArrayMoves(originalArray, currentArray, diff) {
        // Simple move detection for objects with unique identifiers
        const originalItems = originalArray.filter(item => 
            typeof item === 'object' && item !== null && (item.id || item.name || item.uuid)
        );
        
        const currentItems = currentArray.filter(item => 
            typeof item === 'object' && item !== null && (item.id || item.name || item.uuid)
        );

        for (const currentItem of currentItems) {
            const currentId = this.getItemId(currentItem);
            const originalIndex = originalItems.findIndex(item => 
                this.getItemId(item) === currentId
            );
            const currentIndex = currentItems.indexOf(currentItem);

            if (originalIndex !== -1 && originalIndex !== currentIndex) {
                // Mark as moved
                if (diff[currentIndex] && diff[currentIndex].__change_type !== 'added') {
                    diff[currentIndex].__change_type = 'moved';
                    diff[currentIndex].__moved_from = originalIndex;
                    diff[currentIndex].__moved_to = currentIndex;
                }
            }
        }
    }

    getItemId(item) {
        return item.id || item.name || item.uuid || item.key || JSON.stringify(item);
    }

    createUnchangedDiff(value) {
        return {
            __change_type: 'unchanged',
            __current_value: value,
            __original_value: value
        };
    }

    createAddedDiff(value) {
        return {
            __change_type: 'added',
            __current_value: value,
            __original_value: undefined
        };
    }

    createDeletedDiff(value) {
        return {
            __change_type: 'deleted',
            __current_value: undefined,
            __original_value: value
        };
    }

    createModifiedDiff(originalValue, currentValue) {
        return {
            __change_type: 'modified',
            __current_value: currentValue,
            __original_value: originalValue
        };
    }

    createReplacedDiff(originalValue, currentValue) {
        return {
            __change_type: 'replaced',
            __current_value: currentValue,
            __original_value: originalValue
        };
    }

    extractChanges(diff) {
        const changes = new Map();
        this.collectChanges(diff, '', changes);
        return changes;
    }

    collectChanges(obj, path, changes) {
        if (typeof obj !== 'object' || obj === null) {
            return;
        }

        if (obj.__change_type && obj.__change_type !== 'unchanged') {
            changes.set(path, {
                type: obj.__change_type,
                oldValue: obj.__original_value,
                newValue: obj.__current_value,
                path: path
            });
            return;
        }

        // Recursively collect changes from nested objects
        for (const [key, value] of Object.entries(obj)) {
            if (!key.startsWith('__')) {
                const newPath = path ? `${path}.${key}` : key;
                this.collectChanges(value, newPath, changes);
            }
        }
    }

    renderDiff(diff, options = {}) {
        const {
            showUnchanged = true,
            showComments = true,
            sideBySide = false,
            sectionFilter = 'all'
        } = options;

        let html = '';

        for (const [sectionName, sectionDiff] of Object.entries(diff)) {
            if (sectionFilter !== 'all' && sectionFilter !== sectionName) {
                continue;
            }

            const sectionHtml = this.renderSection(
                sectionName, 
                sectionDiff, 
                { showUnchanged, showComments, sideBySide }
            );
            
            if (sectionHtml) {
                html += sectionHtml;
            }
        }

        return html || '<div class="no-diff">No changes to display</div>';
    }

    renderSection(sectionName, sectionDiff, options) {
        const { showUnchanged, showComments, sideBySide } = options;
        
        const changeType = this.getSectionChangeType(sectionDiff);
        
        if (!showUnchanged && changeType === 'unchanged') {
            return '';
        }

        const sectionHtml = this.renderSectionContent(
            sectionDiff, 
            sectionName, 
            options
        );

        if (!sectionHtml) {
            return '';
        }

        return `
            <div class="diff-section ${changeType}">
                <div class="section-header">
                    <h3 class="section-title">
                        <i class="fas ${this.getSectionIcon(changeType)}"></i>
                        ${this.formatSectionName(sectionName)}
                    </h3>
                    <div class="section-stats">
                        ${this.getSectionStats(sectionDiff)}
                    </div>
                </div>
                <div class="section-content ${sideBySide ? 'side-by-side' : 'unified'}">
                    ${sectionHtml}
                </div>
            </div>
        `;
    }

    renderSectionContent(sectionDiff, sectionName, options, indent = 0) {
        if (typeof sectionDiff !== 'object' || sectionDiff === null) {
            return this.renderPrimitiveDiff(sectionDiff, options, indent);
        }

        if (sectionDiff.__change_type) {
            return this.renderChangeItem(sectionDiff, sectionName, options, indent);
        }

        let html = '';
        for (const [key, value] of Object.entries(sectionDiff)) {
            if (key.startsWith('__')) continue;

            const itemHtml = this.renderSectionContent(
                value, 
                key, 
                options, 
                indent + 1
            );
            
            if (itemHtml) {
                html += itemHtml;
            }
        }

        return html;
    }

    renderChangeItem(changeDiff, itemName, options, indent) {
        const { showUnchanged, sideBySide } = options;
        const changeType = changeDiff.__change_type;

        if (!showUnchanged && changeType === 'unchanged') {
            return '';
        }

        const indentClass = `indent-${Math.min(indent, 5)}`;
        
        if (sideBySide) {
            return this.renderSideBySideItem(changeDiff, itemName, indentClass);
        } else {
            return this.renderUnifiedItem(changeDiff, itemName, indentClass);
        }
    }

    renderUnifiedItem(changeDiff, itemName, indentClass) {
        const changeType = changeDiff.__change_type;
        const originalValue = changeDiff.__original_value;
        const currentValue = changeDiff.__current_value;

        let content = '';
        
        switch (changeType) {
            case 'added':
                content = `
                    <div class="diff-line added">
                        <span class="line-marker">+</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(currentValue)}
                        </span>
                    </div>
                `;
                break;
                
            case 'deleted':
                content = `
                    <div class="diff-line deleted">
                        <span class="line-marker">-</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(originalValue)}
                        </span>
                    </div>
                `;
                break;
                
            case 'modified':
            case 'replaced':
                content = `
                    <div class="diff-line deleted">
                        <span class="line-marker">-</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(originalValue)}
                        </span>
                    </div>
                    <div class="diff-line added">
                        <span class="line-marker">+</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(currentValue)}
                        </span>
                    </div>
                `;
                break;
                
            case 'moved':
                content = `
                    <div class="diff-line moved">
                        <span class="line-marker">↕</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(currentValue)}
                            <span class="move-info">
                                (moved from position ${changeDiff.__moved_from} to ${changeDiff.__moved_to})
                            </span>
                        </span>
                    </div>
                `;
                break;
                
            case 'unchanged':
                content = `
                    <div class="diff-line unchanged">
                        <span class="line-marker"> </span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(currentValue)}
                        </span>
                    </div>
                `;
                break;
        }

        return `<div class="diff-item ${changeType} ${indentClass}">${content}</div>`;
    }

    renderSideBySideItem(changeDiff, itemName, indentClass) {
        const changeType = changeDiff.__change_type;
        const originalValue = changeDiff.__original_value;
        const currentValue = changeDiff.__current_value;

        let leftContent = '';
        let rightContent = '';

        switch (changeType) {
            case 'added':
                leftContent = '<div class="empty-line"></div>';
                rightContent = `
                    <div class="diff-line added">
                        <span class="line-marker">+</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(currentValue)}
                        </span>
                    </div>
                `;
                break;
                
            case 'deleted':
                leftContent = `
                    <div class="diff-line deleted">
                        <span class="line-marker">-</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(originalValue)}
                        </span>
                    </div>
                `;
                rightContent = '<div class="empty-line"></div>';
                break;
                
            case 'modified':
            case 'replaced':
                leftContent = `
                    <div class="diff-line deleted">
                        <span class="line-marker">-</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(originalValue)}
                        </span>
                    </div>
                `;
                rightContent = `
                    <div class="diff-line added">
                        <span class="line-marker">+</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(currentValue)}
                        </span>
                    </div>
                `;
                break;
                
            case 'moved':
            case 'unchanged':
                const lineClass = changeType === 'moved' ? 'moved' : 'unchanged';
                const marker = changeType === 'moved' ? '↕' : ' ';
                const content = `
                    <div class="diff-line ${lineClass}">
                        <span class="line-marker">${marker}</span>
                        <span class="line-content">
                            <strong>${itemName}:</strong> ${this.formatValue(currentValue)}
                        </span>
                    </div>
                `;
                leftContent = content;
                rightContent = content;
                break;
        }

        return `
            <div class="diff-item ${changeType} ${indentClass}">
                <div class="side-by-side-container">
                    <div class="diff-left">${leftContent}</div>
                    <div class="diff-right">${rightContent}</div>
                </div>
            </div>
        `;
    }

    renderPrimitiveDiff(value, options, indent) {
        const indentClass = `indent-${Math.min(indent, 5)}`;
        return `
            <div class="diff-item primitive ${indentClass}">
                <div class="diff-line unchanged">
                    <span class="line-marker"> </span>
                    <span class="line-content">${this.formatValue(value)}</span>
                </div>
            </div>
        `;
    }

    getSectionChangeType(sectionDiff) {
        if (!sectionDiff || typeof sectionDiff !== 'object') {
            return 'unchanged';
        }

        if (sectionDiff.__change_type) {
            return sectionDiff.__change_type;
        }

        const changes = this.collectSectionChanges(sectionDiff);
        if (changes.added > 0 || changes.deleted > 0 || changes.modified > 0) {
            return 'modified';
        }

        return 'unchanged';
    }

    collectSectionChanges(obj) {
        const stats = { added: 0, deleted: 0, modified: 0, moved: 0, unchanged: 0 };

        if (typeof obj !== 'object' || obj === null) {
            return stats;
        }

        if (obj.__change_type) {
            stats[obj.__change_type]++;
            return stats;
        }

        for (const value of Object.values(obj)) {
            if (typeof value === 'object' && value !== null) {
                const subStats = this.collectSectionChanges(value);
                for (const [key, count] of Object.entries(subStats)) {
                    stats[key] += count;
                }
            }
        }

        return stats;
    }

    getSectionStats(sectionDiff) {
        const stats = this.collectSectionChanges(sectionDiff);
        const total = Object.values(stats).reduce((sum, count) => sum + count, 0);
        
        if (total === 0) {
            return '<span class="no-changes">No changes</span>';
        }

        const statItems = [];
        if (stats.added > 0) statItems.push(`<span class="stat-added">+${stats.added}</span>`);
        if (stats.deleted > 0) statItems.push(`<span class="stat-deleted">-${stats.deleted}</span>`);
        if (stats.modified > 0) statItems.push(`<span class="stat-modified">~${stats.modified}</span>`);
        if (stats.moved > 0) statItems.push(`<span class="stat-moved">↕${stats.moved}</span>`);

        return statItems.join(' ');
    }

    getSectionIcon(changeType) {
        const icons = {
            added: 'plus-circle',
            deleted: 'minus-circle',
            modified: 'edit',
            moved: 'arrows-alt',
            unchanged: 'check-circle'
        };
        return icons[changeType] || 'circle';
    }

    formatSectionName(name) {
        return name.charAt(0).toUpperCase() + name.slice(1).replace(/([A-Z])/g, ' $1');
    }

    formatValue(value) {
        if (value === null) return '<span class="null-value">null</span>';
        if (value === undefined) return '<span class="undefined-value">undefined</span>';
        if (value === '') return '<span class="empty-value">""</span>';
        if (typeof value === 'boolean') return `<span class="boolean-value">${value}</span>`;
        if (typeof value === 'number') return `<span class="number-value">${value}</span>`;
        if (typeof value === 'string') {
            // Escape HTML and handle long strings
            const escaped = this.escapeHtml(value);
            if (escaped.length > 100) {
                return `<span class="string-value" title="${escaped}">${escaped.substring(0, 97)}...</span>`;
            }
            return `<span class="string-value">${escaped}</span>`;
        }
        if (Array.isArray(value)) {
            return `<span class="array-value">[${value.length} items]</span>`;
        }
        if (typeof value === 'object') {
            const keys = Object.keys(value);
            return `<span class="object-value">{${keys.length} properties}</span>`;
        }
        return `<span class="unknown-value">${String(value)}</span>`;
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    isEqual(a, b) {
        if (a === b) return true;
        
        if (a === null || b === null) return a === b;
        if (a === undefined || b === undefined) return a === b;
        
        if (typeof a !== typeof b) return false;
        
        if (typeof a === 'number' && typeof b === 'number') {
            return Math.abs(a - b) < this.diffOptions.precision;
        }
        
        if (typeof a === 'string' && typeof b === 'string') {
            if (this.diffOptions.ignoreCase) {
                return a.toLowerCase() === b.toLowerCase();
            }
            if (this.diffOptions.ignoreWhitespace) {
                return a.trim() === b.trim();
            }
            return a === b;
        }
        
        if (Array.isArray(a) && Array.isArray(b)) {
            if (a.length !== b.length) return false;
            for (let i = 0; i < a.length; i++) {
                if (!this.isEqual(a[i], b[i])) return false;
            }
            return true;
        }
        
        if (typeof a === 'object' && typeof b === 'object') {
            const keysA = Object.keys(a);
            const keysB = Object.keys(b);
            
            if (keysA.length !== keysB.length) return false;
            
            for (const key of keysA) {
                if (!keysB.includes(key)) return false;
                if (!this.isEqual(a[key], b[key])) return false;
            }
            return true;
        }
        
        return false;
    }

    isPrimitive(value) {
        return value === null || 
               value === undefined || 
               typeof value === 'string' || 
               typeof value === 'number' || 
               typeof value === 'boolean';
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ConfigDiffEngine;
}
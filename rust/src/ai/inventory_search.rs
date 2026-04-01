use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::ai::content_creation::{ContentItem, ContentCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySearchEngine {
    /// JSON catalog data
    pub catalog: ContentCatalog,
    /// XML search index for fast lookups
    pub search_index: XMLSearchIndex,
    /// Search configuration and weights
    pub config: SearchConfig,
    /// Performance metrics
    pub metrics: SearchMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCatalog {
    pub metadata: CatalogMetadata,
    pub categories: HashMap<String, CategoryInfo>,
    pub items: HashMap<String, ContentItem>,
    pub patterns: HashMap<String, PatternInfo>,
    pub search_optimization: SearchOptimization,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogMetadata {
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub system: String,
    pub description: String,
    pub total_items: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    pub name: String,
    pub description: String,
    pub complexity_range: (f64, f64),
    pub subcategories: Vec<String>,
    pub item_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInfo {
    pub characteristics: Vec<String>,
    pub applicable_items: Vec<String>,
    pub quality_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptimization {
    pub indexed_fields: Vec<String>,
    pub search_weights: HashMap<String, f64>,
    pub common_queries: HashMap<String, Vec<String>>,
    pub category_mappings: HashMap<String, Vec<String>>,
    pub complexity_ranges: HashMap<String, ComplexityRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityRange {
    pub level: String,
    pub min: f64,
    pub max: f64,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub scoring_criteria: HashMap<String, f64>,
    pub minimum_quality_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMLSearchIndex {
    pub version: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub total_items: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub items: Vec<IndexedItem>,
    pub search_optimization: IndexOptimization,
    pub pattern_recognition: PatternRecognition,
    pub eads_metadata: EADSMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedItem {
    pub id: String,
    pub category: String,
    pub subcategory: String,
    pub complexity: f64,
    pub quality: f64,
    pub tags: Vec<String>,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexOptimization {
    pub common_queries: HashMap<String, Vec<String>>,
    pub category_mappings: HashMap<String, Vec<String>>,
    pub complexity_ranges: HashMap<String, ComplexityRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRecognition {
    pub learned_patterns: HashMap<String, LearnedPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub characteristics: Vec<String>,
    pub applicable_items: Vec<String>,
    pub quality_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EADSMetadata {
    pub learning_iterations: u32,
    pub pattern_recognition_accuracy: f64,
    pub quality_improvement_rate: f64,
    pub user_satisfaction_score: f64,
    pub content_generation_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub fuzzy_search: bool,
    pub fuzzy_threshold: f64,
    pub max_results: usize,
    pub enable_suggestions: bool,
    pub enable_autocomplete: bool,
    pub search_weights: SearchWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWeights {
    pub name: f64,
    pub tags: f64,
    pub description: f64,
    pub category: f64,
    pub keywords: f64,
    pub attributes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetrics {
    pub total_searches: u64,
    pub successful_searches: u64,
    pub average_response_time_ms: f64,
    pub most_searched_terms: HashMap<String, u32>,
    pub search_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub items: Vec<SearchResultItem>,
    pub total_matches: usize,
    pub search_time_ms: f64,
    pub suggestions: Vec<String>,
    pub related_queries: Vec<String>,
    pub query_understanding: QueryUnderstanding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub item: ContentItem,
    pub relevance_score: f64,
    pub match_reasons: Vec<MatchReason>,
    pub quality_score: f64,
    pub complexity_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReason {
    pub field: String,
    pub matched_term: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryUnderstanding {
    pub intent: SearchIntent,
    pub complexity_preference: Option<String>,
    pub category_preference: Option<String>,
    pub extracted_keywords: Vec<String>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchIntent {
    FindSpecificItem,
    BrowseCategory,
    FindByComplexity,
    FindSimilar,
    GenerateIdeas,
    LearnPattern,
}

impl InventorySearchEngine {
    pub fn new(catalog_path: &Path, index_path: &Path) -> Result<Self> {
        let catalog = Self::load_catalog(catalog_path)?;
        let search_index = Self::load_search_index(index_path)?;
        
        Ok(Self {
            catalog,
            search_index,
            config: SearchConfig::default(),
            metrics: SearchMetrics::new(),
        })
    }

    /// Perform instant content search with recognition and conclusion capabilities
    pub async fn search(&mut self, query: &str) -> Result<SearchResult> {
        let start_time = std::time::Instant::now();
        
        tracing::info!("Performing search for: '{}'", query);
        
        // Parse and understand the query
        let query_understanding = self.understand_query(query).await?;
        
        // Perform the actual search
        let mut matches = self.find_matches(query, &query_understanding).await?;
        
        // Rank and filter results
        self.rank_results(&mut matches, &query_understanding).await?;
        
        // Limit results
        matches.truncate(self.config.max_results);
        
        // Generate suggestions and related queries
        let suggestions = self.generate_suggestions(query, &matches).await?;
        let related_queries = self.generate_related_queries(&query_understanding).await?;
        
        let search_time = start_time.elapsed().as_millis() as f64;
        
        // Update metrics
        self.update_search_metrics(query, &matches, search_time).await?;
        
        let result = SearchResult {
            items: matches,
            total_matches: matches.len(),
            search_time_ms: search_time,
            suggestions,
            related_queries,
            query_understanding,
        };
        
        tracing::info!("Search completed in {}ms with {} results", search_time, result.items.len());
        Ok(result)
    }

    /// Perform XML-based fast lookup for instant recognition
    pub async fn xml_search(&self, query: &str) -> Result<Vec<String>> {
        tracing::debug!("Performing XML search for: '{}'", query);
        
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        // Search through indexed items
        for item in &self.search_index.items {
            let mut relevance = 0.0;
            
            // Check name match
            if item.name.to_lowercase().contains(&query_lower) {
                relevance += 1.0;
            }
            
            // Check tag matches
            for tag in &item.tags {
                if tag.to_lowercase().contains(&query_lower) {
                    relevance += 0.8;
                }
            }
            
            // Check keyword matches
            for keyword in &item.keywords {
                if keyword.to_lowercase().contains(&query_lower) {
                    relevance += 0.6;
                }
            }
            
            // Check category match
            if item.category.to_lowercase().contains(&query_lower) {
                relevance += 0.4;
            }
            
            // Check description match
            if item.description.to_lowercase().contains(&query_lower) {
                relevance += 0.3;
            }
            
            if relevance > 0.0 {
                results.push(item.id.clone());
            }
        }
        
        // Check common queries optimization
        if let Some(optimized_results) = self.search_index.search_optimization.common_queries.get(query) {
            results.extend(optimized_results.clone());
        }
        
        // Remove duplicates and sort by relevance
        results.sort();
        results.dedup();
        
        tracing::debug!("XML search found {} results", results.len());
        Ok(results)
    }

    /// Get content by category with filtering options
    pub async fn get_by_category(
        &self,
        category: &str,
        filters: Option<CategoryFilters>,
    ) -> Result<Vec<ContentItem>> {
        let mut results = Vec::new();
        
        // Get all items in category
        for (item_id, item) in &self.catalog.items {
            if format!("{:?}", item.category).to_lowercase() == category.to_lowercase() {
                let mut include = true;
                
                // Apply filters if provided
                if let Some(ref filters) = filters {
                    if let Some(min_complexity) = filters.min_complexity {
                        if item.metadata.complexity_score < min_complexity {
                            include = false;
                        }
                    }
                    
                    if let Some(max_complexity) = filters.max_complexity {
                        if item.metadata.complexity_score > max_complexity {
                            include = false;
                        }
                    }
                    
                    if let Some(ref required_tags) = filters.required_tags {
                        let item_tags: Vec<String> = item.metadata.tags.clone();
                        for required_tag in required_tags {
                            if !item_tags.contains(required_tag) {
                                include = false;
                                break;
                            }
                        }
                    }
                }
                
                if include {
                    results.push(item.clone());
                }
            }
        }
        
        // Sort by quality score
        results.sort_by(|a, b| {
            b.metadata.quality_score.partial_cmp(&a.metadata.quality_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(results)
    }

    /// Get similar items based on patterns and characteristics
    pub async fn get_similar_items(&self, item_id: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let target_item = self.catalog.items.get(item_id)
            .ok_or_else(|| anyhow::anyhow!("Item not found: {}", item_id))?;
        
        let mut similar_items = Vec::new();
        
        for (other_id, other_item) in &self.catalog.items {
            if other_id == item_id {
                continue; // Skip the target item itself
            }
            
            let similarity_score = self.calculate_similarity(target_item, other_item).await?;
            
            if similarity_score > 0.3 { // Minimum similarity threshold
                similar_items.push((other_item.clone(), similarity_score));
            }
        }
        
        // Sort by similarity score
        similar_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top matches
        Ok(similar_items.into_iter()
            .take(limit)
            .map(|(item, _)| item)
            .collect())
    }

    /// Generate autocomplete suggestions
    pub async fn autocomplete(&self, partial_query: &str, limit: usize) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        let partial_lower = partial_query.to_lowercase();
        
        // Search through item names
        for item in self.catalog.items.values() {
            if item.name.to_lowercase().starts_with(&partial_lower) {
                suggestions.push(item.name.clone());
            }
        }
        
        // Search through tags
        for item in self.catalog.items.values() {
            for tag in &item.metadata.tags {
                if tag.to_lowercase().starts_with(&partial_lower) {
                    suggestions.push(tag.clone());
                }
            }
        }
        
        // Search through categories
        for category_name in self.catalog.categories.keys() {
            if category_name.to_lowercase().starts_with(&partial_lower) {
                suggestions.push(category_name.clone());
            }
        }
        
        // Remove duplicates and sort
        suggestions.sort();
        suggestions.dedup();
        
        // Limit results
        suggestions.truncate(limit);
        
        Ok(suggestions)
    }

    /// Update search index with new content
    pub async fn update_index(&mut self, new_items: Vec<ContentItem>) -> Result<()> {
        tracing::info!("Updating search index with {} new items", new_items.len());
        
        for item in new_items {
            // Add to catalog
            self.catalog.items.insert(item.id.to_string(), item.clone());
            
            // Create indexed version
            let indexed_item = self.create_indexed_item(&item).await?;
            
            // Add to search index
            self.search_index.items.push(indexed_item);
        }
        
        // Update metadata
        self.catalog.metadata.total_items = self.catalog.items.len() as u32;
        self.catalog.metadata.last_updated = chrono::Utc::now();
        self.search_index.total_items = self.search_index.items.len() as u32;
        self.search_index.last_updated = chrono::Utc::now();
        
        tracing::info!("Search index updated successfully");
        Ok(())
    }

    /// Save catalog and index to files
    pub async fn save(&self, catalog_path: &Path, index_path: &Path) -> Result<()> {
        // Save JSON catalog
        let catalog_file = File::create(catalog_path)
            .with_context(|| format!("Failed to create catalog file: {}", catalog_path.display()))?;
        serde_json::to_writer_pretty(catalog_file, &self.catalog)
            .with_context(|| "Failed to write catalog JSON")?;
        
        // Save XML index (would implement XML serialization)
        // For now, save as JSON for simplicity
        let index_file = File::create(index_path.with_extension("json"))
            .with_context(|| format!("Failed to create index file: {}", index_path.display()))?;
        serde_json::to_writer_pretty(index_file, &self.search_index)
            .with_context(|| "Failed to write search index")?;
        
        tracing::info!("Inventory search engine saved successfully");
        Ok(())
    }

    // Private helper methods
    fn load_catalog(catalog_path: &Path) -> Result<ContentCatalog> {
        let file = File::open(catalog_path)
            .with_context(|| format!("Failed to open catalog file: {}", catalog_path.display()))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .with_context(|| "Failed to parse catalog JSON")
    }

    fn load_search_index(index_path: &Path) -> Result<XMLSearchIndex> {
        // Try JSON first, then XML
        if let Ok(file) = File::open(index_path.with_extension("json")) {
            let reader = BufReader::new(file);
            return serde_json::from_reader(reader)
                .with_context(|| "Failed to parse search index JSON");
        }
        
        // XML parsing would go here
        // For now, return empty index
        Ok(XMLSearchIndex::default())
    }

    async fn understand_query(&self, query: &str) -> Result<QueryUnderstanding> {
        let mut intent = SearchIntent::FindSpecificItem;
        let mut complexity_preference = None;
        let mut category_preference = None;
        let extracted_keywords = self.extract_keywords(query);
        
        // Analyze query for intent and preferences
        let query_lower = query.to_lowercase();
        
        if query_lower.contains("browse") || query_lower.contains("show me") {
            intent = SearchIntent::BrowseCategory;
        } else if query_lower.contains("similar") || query_lower.contains("like") {
            intent = SearchIntent::FindSimilar;
        } else if query_lower.contains("beginner") || query_lower.contains("simple") {
            complexity_preference = Some("beginner".to_string());
        } else if query_lower.contains("advanced") || query_lower.contains("complex") {
            complexity_preference = Some("advanced".to_string());
        }
        
        // Check for category mentions
        for category_name in self.catalog.categories.keys() {
            if query_lower.contains(&category_name.to_lowercase()) {
                category_preference = Some(category_name.clone());
                break;
            }
        }
        
        let confidence_score = self.calculate_query_confidence(
            &intent,
            &complexity_preference,
            &category_preference,
            &extracted_keywords,
            query,
        );

        Ok(QueryUnderstanding {
            intent,
            complexity_preference,
            category_preference,
            extracted_keywords,
            confidence_score,
        })
    }

    fn extract_keywords(&self, query: &str) -> Vec<String> {
        query.split_whitespace()
            .map(|word| word.to_lowercase())
            .filter(|word| word.len() > 2) // Filter out very short words
            .collect()
    }

    fn calculate_query_confidence(
        &self,
        intent: &SearchIntent,
        complexity_preference: &Option<String>,
        category_preference: &Option<String>,
        extracted_keywords: &[String],
        query: &str,
    ) -> f64 {
        let mut confidence = 0.5;

        match intent {
            SearchIntent::Specific => confidence += 0.2,
            SearchIntent::Browsing => confidence += 0.1,
            SearchIntent::Similar => confidence += 0.15,
            SearchIntent::Exploratory => confidence += 0.05,
        }

        if complexity_preference.is_some() {
            confidence += 0.1;
        }

        if category_preference.is_some() {
            confidence += 0.15;
        }

        let keyword_factor = (extracted_keywords.len() as f64 * 0.02).min(0.15);
        confidence += keyword_factor;

        let query_length_factor = if query.len() > 5 && query.len() < 100 {
            0.05
        } else {
            0.0
        };
        confidence += query_length_factor;

        confidence.min(1.0)
    }

    async fn find_matches(&self, query: &str, understanding: &QueryUnderstanding) -> Result<Vec<SearchResultItem>> {
        let mut matches = Vec::new();
        
        for item in self.catalog.items.values() {
            let relevance_score = self.calculate_relevance(item, query, understanding).await?;
            
            if relevance_score > 0.1 { // Minimum relevance threshold
                let match_reasons = self.get_match_reasons(item, query);
                let complexity_level = self.get_complexity_level(item.metadata.complexity_score);
                
                matches.push(SearchResultItem {
                    item: item.clone(),
                    relevance_score,
                    match_reasons,
                    quality_score: item.metadata.quality_score,
                    complexity_level,
                });
            }
        }
        
        Ok(matches)
    }

    async fn calculate_relevance(&self, item: &ContentItem, query: &str, understanding: &QueryUnderstanding) -> Result<f64> {
        let mut score = 0.0;
        let query_lower = query.to_lowercase();
        
        // Name match
        if item.name.to_lowercase().contains(&query_lower) {
            score += self.config.search_weights.name;
        }
        
        // Tag matches
        for tag in &item.metadata.tags {
            if tag.to_lowercase().contains(&query_lower) {
                score += self.config.search_weights.tags;
            }
        }
        
        // Description match
        if item.description.to_lowercase().contains(&query_lower) {
            score += self.config.search_weights.description;
        }
        
        // Category match
        if format!("{:?}", item.category).to_lowercase().contains(&query_lower) {
            score += self.config.search_weights.category;
        }
        
        // Keyword matches
        for keyword in &understanding.extracted_keywords {
            if item.name.to_lowercase().contains(keyword) ||
               item.description.to_lowercase().contains(keyword) ||
               item.metadata.tags.iter().any(|tag| tag.to_lowercase().contains(keyword)) {
                score += self.config.search_weights.keywords;
            }
        }
        
        // Boost for category preference
        if let Some(ref preferred_category) = understanding.category_preference {
            if format!("{:?}", item.category).to_lowercase() == preferred_category.to_lowercase() {
                score *= 1.5;
            }
        }
        
        // Boost for complexity preference
        if let Some(ref complexity_pref) = understanding.complexity_preference {
            let item_complexity_level = self.get_complexity_level(item.metadata.complexity_score);
            if item_complexity_level == *complexity_pref {
                score *= 1.3;
            }
        }
        
        Ok(score)
    }

    async fn rank_results(&self, matches: &mut Vec<SearchResultItem>, understanding: &QueryUnderstanding) -> Result<()> {
        // Sort by combined score of relevance and quality
        matches.sort_by(|a, b| {
            let score_a = a.relevance_score * 0.7 + a.quality_score * 0.3;
            let score_b = b.relevance_score * 0.7 + b.quality_score * 0.3;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(())
    }

    fn get_match_reasons(&self, item: &ContentItem, query: &str) -> Vec<MatchReason> {
        let mut reasons = Vec::new();
        let query_lower = query.to_lowercase();
        
        if item.name.to_lowercase().contains(&query_lower) {
            reasons.push(MatchReason {
                field: "name".to_string(),
                matched_term: query.to_string(),
                score: 1.0,
            });
        }
        
        for tag in &item.metadata.tags {
            if tag.to_lowercase().contains(&query_lower) {
                reasons.push(MatchReason {
                    field: "tags".to_string(),
                    matched_term: tag.clone(),
                    score: 0.8,
                });
            }
        }
        
        reasons
    }

    fn get_complexity_level(&self, complexity_score: f64) -> String {
        if complexity_score <= 3.0 {
            "beginner".to_string()
        } else if complexity_score <= 6.0 {
            "intermediate".to_string()
        } else if complexity_score <= 8.5 {
            "advanced".to_string()
        } else {
            "expert".to_string()
        }
    }

    async fn generate_suggestions(&self, query: &str, matches: &[SearchResultItem]) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        
        // Generate suggestions based on query and results
        if matches.is_empty() {
            suggestions.push("Try a more general search term".to_string());
            suggestions.push("Browse categories instead".to_string());
        } else {
            // Suggest related terms based on found items
            for item in matches.iter().take(3) {
                for tag in &item.item.metadata.tags {
                    if !tag.to_lowercase().contains(&query.to_lowercase()) {
                        suggestions.push(format!("Search for '{}'", tag));
                    }
                }
            }
        }
        
        suggestions.truncate(5);
        Ok(suggestions)
    }

    async fn generate_related_queries(&self, understanding: &QueryUnderstanding) -> Result<Vec<String>> {
        let mut related = Vec::new();
        
        // Generate related queries based on understanding
        if let Some(ref category) = understanding.category_preference {
            related.push(format!("Browse all {}", category));
            related.push(format!("Advanced {}", category));
        }
        
        for keyword in &understanding.extracted_keywords {
            related.push(format!("Similar to {}", keyword));
        }
        
        related.truncate(5);
        Ok(related)
    }

    async fn calculate_similarity(&self, item1: &ContentItem, item2: &ContentItem) -> Result<f64> {
        let mut similarity = 0.0;
        
        // Category similarity
        if item1.category == item2.category {
            similarity += 0.3;
        }
        
        // Tag similarity
        let common_tags = item1.metadata.tags.iter()
            .filter(|tag| item2.metadata.tags.contains(tag))
            .count();
        let total_tags = (item1.metadata.tags.len() + item2.metadata.tags.len()) as f64;
        if total_tags > 0.0 {
            similarity += (common_tags as f64 / total_tags) * 0.4;
        }
        
        // Complexity similarity
        let complexity_diff = (item1.metadata.complexity_score - item2.metadata.complexity_score).abs();
        if complexity_diff < 2.0 {
            similarity += (2.0 - complexity_diff) / 2.0 * 0.3;
        }
        
        Ok(similarity)
    }

    async fn create_indexed_item(&self, item: &ContentItem) -> Result<IndexedItem> {
        Ok(IndexedItem {
            id: item.id.to_string(),
            category: format!("{:?}", item.category),
            subcategory: item.metadata.subcategory.clone().unwrap_or_default(),
            complexity: item.metadata.complexity_score,
            quality: item.metadata.quality_score,
            tags: item.metadata.tags.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            keywords: self.extract_keywords(&format!("{} {}", item.name, item.description)),
            attributes: HashMap::new(), // Would extract from metadata
            patterns: Vec::new(), // Would extract from patterns
        })
    }

    async fn update_search_metrics(&mut self, query: &str, matches: &[SearchResultItem], search_time: f64) -> Result<()> {
        self.metrics.total_searches += 1;
        
        if !matches.is_empty() {
            self.metrics.successful_searches += 1;
        }
        
        // Update average response time
        let total_time = self.metrics.average_response_time_ms * (self.metrics.total_searches - 1) as f64 + search_time;
        self.metrics.average_response_time_ms = total_time / self.metrics.total_searches as f64;
        
        // Track search terms
        *self.metrics.most_searched_terms.entry(query.to_string()).or_insert(0) += 1;
        
        // Calculate success rate
        self.metrics.search_success_rate = self.metrics.successful_searches as f64 / self.metrics.total_searches as f64;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryFilters {
    pub min_complexity: Option<f64>,
    pub max_complexity: Option<f64>,
    pub required_tags: Option<Vec<String>>,
    pub min_quality: Option<f64>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            fuzzy_search: true,
            fuzzy_threshold: 0.8,
            max_results: 50,
            enable_suggestions: true,
            enable_autocomplete: true,
            search_weights: SearchWeights {
                name: 1.0,
                tags: 0.8,
                description: 0.6,
                category: 0.4,
                keywords: 0.7,
                attributes: 0.3,
            },
        }
    }
}

impl Default for XMLSearchIndex {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            created: chrono::Utc::now(),
            total_items: 0,
            last_updated: chrono::Utc::now(),
            items: Vec::new(),
            search_optimization: IndexOptimization {
                common_queries: HashMap::new(),
                category_mappings: HashMap::new(),
                complexity_ranges: HashMap::new(),
            },
            pattern_recognition: PatternRecognition {
                learned_patterns: HashMap::new(),
            },
            eads_metadata: EADSMetadata {
                learning_iterations: 0,
                pattern_recognition_accuracy: 0.0,
                quality_improvement_rate: 0.0,
                user_satisfaction_score: 0.0,
                content_generation_success_rate: 0.0,
            },
        }
    }
}

impl SearchMetrics {
    pub fn new() -> Self {
        Self {
            total_searches: 0,
            successful_searches: 0,
            average_response_time_ms: 0.0,
            most_searched_terms: HashMap::new(),
            search_success_rate: 0.0,
        }
    }
}
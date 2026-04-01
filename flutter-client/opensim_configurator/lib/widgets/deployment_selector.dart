// Deployment Selector Widget - Flutter Web Version
// Intelligent deployment type selection with auto-detection and recommendations

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';
import '../models/deployment_models.dart';
import '../theme/app_theme.dart';

class DeploymentSelector extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Consumer<ConfiguratorProvider>(
      builder: (context, provider, child) {
        return Card(
          child: Padding(
            padding: EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Choose Deployment Type',
                  style: AppTheme.headlineStyle(),
                ),
                SizedBox(height: 8),
                Text(
                  'Select the type of OpenSim Next deployment that best matches your needs.',
                  style: AppTheme.bodyStyle(),
                ),
                SizedBox(height: 24),
                
                // Auto-Detection Panel
                _buildAutoDetectionPanel(provider),
                
                SizedBox(height: 24),
                
                // Deployment Options
                _buildDeploymentOptions(provider),
                
                SizedBox(height: 24),
                
                // Comparison Matrix (collapsible)
                _buildComparisonSection(provider),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildAutoDetectionPanel(ConfiguratorProvider provider) {
    return Container(
      padding: EdgeInsets.all(20),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            AppTheme.primaryColor.withValues(alpha: 0.1),
            AppTheme.primaryLight.withValues(alpha: 0.05),
          ],
        ),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.primaryColor.withValues(alpha: 0.2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.auto_fix_high, color: AppTheme.primaryColor, size: 24),
              SizedBox(width: 12),
              Text(
                '🧠 Smart Deployment Detection',
                style: AppTheme.titleStyle(),
              ),
            ],
          ),
          SizedBox(height: 8),
          Text(
            'Our intelligent system analyzes your environment to recommend the optimal deployment type',
            style: AppTheme.bodyStyle(),
          ),
          SizedBox(height: 16),
          
          if (provider.autoDetectionResults != null) ...[
            _buildDetectionResults(provider.autoDetectionResults!),
          ] else ...[
            ElevatedButton.icon(
              onPressed: () => provider.runAutoDetection(),
              icon: Icon(Icons.psychology),
              label: Text('Analyze My Environment'),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildDetectionResults(AutoDetectionResults results) {
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppTheme.successColor.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.check_circle, color: AppTheme.successColor, size: 20),
              SizedBox(width: 8),
              Text(
                'Recommended: ${results.recommendedDeployment}',
                style: TextStyle(
                  fontWeight: FontWeight.w600,
                  color: AppTheme.successColor,
                ),
              ),
            ],
          ),
          SizedBox(height: 8),
          Text(
            results.reasoning,
            style: AppTheme.bodyStyle(),
          ),
          SizedBox(height: 12),
          
          // Confidence Meter
          Row(
            children: [
              Text('Confidence: ', style: AppTheme.captionStyle()),
              Expanded(
                child: LinearProgressIndicator(
                  value: results.confidence,
                  backgroundColor: AppTheme.gray200,
                  valueColor: AlwaysStoppedAnimation<Color>(
                    results.confidence > 0.8 ? AppTheme.successColor :
                    results.confidence > 0.6 ? AppTheme.warningColor :
                    AppTheme.errorColor
                  ),
                ),
              ),
              SizedBox(width: 8),
              Text('${(results.confidence * 100).toInt()}%', 
                   style: AppTheme.captionStyle()),
            ],
          ),
          
          SizedBox(height: 16),
          
          // Detection Details
          ExpansionTile(
            title: Text('Detection Analysis', style: AppTheme.bodyStyle()),
            tilePadding: EdgeInsets.zero,
            children: [
              Padding(
                padding: EdgeInsets.only(top: 8),
                child: Column(
                  children: results.analysisDetails.map((detail) =>
                    Padding(
                      padding: EdgeInsets.symmetric(vertical: 2),
                      child: Row(
                        children: [
                          Icon(Icons.circle, size: 6, color: AppTheme.gray400),
                          SizedBox(width: 8),
                          Expanded(child: Text(detail, style: AppTheme.captionStyle())),
                        ],
                      ),
                    )
                  ).toList(),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildDeploymentOptions(ConfiguratorProvider provider) {
    return Column(
      children: [
        _buildDeploymentOption(
          provider,
          DeploymentType.Development,
          'Development Environment',
          'Optimized for rapid development, testing, and learning',
          Icons.laptop,
          AppTheme.developmentColor,
          [
            'SQLite database (no setup required)',
            'ODE physics engine (stable)',
            'Local network only',
            'Basic security for development',
            '15-30 minute setup time'
          ],
          {
            'Users': '1-10 concurrent',
            'Regions': '1-4 regions',
            'Hardware': '4 cores, 8GB RAM'
          },
        ),
        SizedBox(height: 16),
        _buildDeploymentOption(
          provider,
          DeploymentType.Production,
          'Production Environment',
          'Battle-tested configuration for live virtual worlds',
          Icons.business,
          AppTheme.productionColor,
          [
            'PostgreSQL database (high performance)',
            'Bullet/UBODE physics engines',
            'SSL/TLS security',
            'Professional monitoring',
            '2-4 hour setup time'
          ],
          {
            'Users': '10-500 concurrent',
            'Regions': '4-32 regions',
            'Hardware': '16 cores, 32GB RAM'
          },
        ),
        SizedBox(height: 16),
        _buildDeploymentOption(
          provider,
          DeploymentType.Grid,
          'Grid Environment',
          'Distributed multi-server architecture for massive scale',
          Icons.grid_on,
          AppTheme.gridColor,
          [
            'PostgreSQL clustering',
            'POS physics with GPU acceleration',
            'Zero trust networking (OpenZiti)',
            'Enterprise security & monitoring',
            '1-2 day setup time'
          ],
          {
            'Users': '100-10,000+ concurrent',
            'Regions': '32-1000+ regions',
            'Hardware': '64+ cores, 128GB+ RAM'
          },
        ),
      ],
    );
  }

  Widget _buildDeploymentOption(
    ConfiguratorProvider provider,
    DeploymentType type,
    String title,
    String description,
    IconData icon,
    Color color,
    List<String> features,
    Map<String, String> specs,
  ) {
    final isSelected = provider.selectedDeploymentType == type;
    final isRecommended = provider.autoDetectionResults?.recommendedDeployment == title;

    return Container(
      decoration: BoxDecoration(
        border: Border.all(
          color: isSelected ? color : AppTheme.gray300,
          width: isSelected ? 2 : 1,
        ),
        borderRadius: BorderRadius.circular(12),
        color: isSelected ? color.withValues(alpha: 0.05) : Colors.white,
      ),
      child: InkWell(
        onTap: () => provider.selectDeploymentType(type),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    padding: EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: color.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Icon(icon, color: color, size: 24),
                  ),
                  SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Text(title, style: AppTheme.titleStyle()),
                            if (isRecommended) ...[
                              SizedBox(width: 8),
                              Container(
                                padding: EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                                decoration: BoxDecoration(
                                  color: AppTheme.successColor,
                                  borderRadius: BorderRadius.circular(12),
                                ),
                                child: Text(
                                  'RECOMMENDED',
                                  style: TextStyle(
                                    color: Colors.white,
                                    fontSize: 10,
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                              ),
                            ],
                          ],
                        ),
                        SizedBox(height: 4),
                        Text(description, style: AppTheme.bodyStyle()),
                      ],
                    ),
                  ),
                  Radio<DeploymentType>(
                    value: type,
                    groupValue: provider.selectedDeploymentType,
                    onChanged: (value) => provider.selectDeploymentType(value!),
                    activeColor: color,
                  ),
                ],
              ),
              
              SizedBox(height: 16),
              
              // Features
              Text('Features:', style: AppTheme.captionStyle()),
              SizedBox(height: 8),
              ...features.map((feature) => Padding(
                padding: EdgeInsets.symmetric(vertical: 2),
                child: Row(
                  children: [
                    Icon(Icons.check, color: color, size: 16),
                    SizedBox(width: 8),
                    Expanded(child: Text(feature, style: AppTheme.bodyStyle())),
                  ],
                ),
              )).toList(),
              
              SizedBox(height: 16),
              
              // Specifications
              Container(
                padding: EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: AppTheme.gray50,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  children: specs.entries.map((spec) =>
                    Padding(
                      padding: EdgeInsets.symmetric(vertical: 2),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(spec.key, style: AppTheme.captionStyle()),
                          Text(spec.value, style: AppTheme.bodyStyle()),
                        ],
                      ),
                    )
                  ).toList(),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildComparisonSection(ConfiguratorProvider provider) {
    return ExpansionTile(
      title: Text('Detailed Comparison Matrix', style: AppTheme.titleStyle()),
      tilePadding: EdgeInsets.zero,
      children: [
        Container(
          padding: EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: AppTheme.gray50,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Table(
            border: TableBorder.all(color: AppTheme.gray300),
            columnWidths: {
              0: FlexColumnWidth(2),
              1: FlexColumnWidth(1),
              2: FlexColumnWidth(1),
              3: FlexColumnWidth(1),
            },
            children: [
              // Header
              TableRow(
                decoration: BoxDecoration(color: AppTheme.gray100),
                children: [
                  _buildTableCell('Feature', isHeader: true),
                  _buildTableCell('Development', isHeader: true),
                  _buildTableCell('Production', isHeader: true),
                  _buildTableCell('Grid', isHeader: true),
                ],
              ),
              // Data rows
              ..._comparisonData.map((row) => TableRow(
                children: [
                  _buildTableCell(row['feature']!),
                  _buildTableCell(row['development']!),
                  _buildTableCell(row['production']!),
                  _buildTableCell(row['grid']!),
                ],
              )).toList(),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildTableCell(String text, {bool isHeader = false}) {
    return Padding(
      padding: EdgeInsets.all(8),
      child: Text(
        text,
        style: isHeader ? AppTheme.captionStyle() : AppTheme.bodyStyle(),
        textAlign: TextAlign.center,
      ),
    );
  }
}

// Comparison data for the table
final List<Map<String, String>> _comparisonData = [
  {
    'feature': 'Database',
    'development': 'SQLite',
    'production': 'PostgreSQL',
    'grid': 'PostgreSQL Cluster'
  },
  {
    'feature': 'Physics Engine',
    'development': 'ODE',
    'production': 'Bullet/UBODE',
    'grid': 'POS/GPU'
  },
  {
    'feature': 'Security',
    'development': 'Basic',
    'production': 'SSL/TLS',
    'grid': 'Zero Trust'
  },
  {
    'feature': 'Monitoring',
    'development': 'Simple',
    'production': 'Professional',
    'grid': 'Enterprise'
  },
  {
    'feature': 'Setup Time',
    'development': '15-30 min',
    'production': '2-4 hours',
    'grid': '1-2 days'
  },
];

// Data models
class AutoDetectionResults {
  final String recommendedDeployment;
  final String reasoning;
  final double confidence;
  final List<String> analysisDetails;

  AutoDetectionResults({
    required this.recommendedDeployment,
    required this.reasoning,
    required this.confidence,
    required this.analysisDetails,
  });
}
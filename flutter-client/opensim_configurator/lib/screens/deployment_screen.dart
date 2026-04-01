// OpenSim Next Configurator - Deployment Screen
// Auto-detection and deployment type selection

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';
import '../models/deployment_models.dart';
import '../theme/app_theme.dart';
import '../widgets/system_info_form.dart';

class DeploymentScreen extends StatefulWidget {
  @override
  _DeploymentScreenState createState() => _DeploymentScreenState();
}

class _DeploymentScreenState extends State<DeploymentScreen> with TickerProviderStateMixin {
  late TabController _tabController;
  final _systemInfoFormKey = GlobalKey<FormState>();

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Deployment Type'),
        bottom: TabBar(
          controller: _tabController,
          tabs: [
            Tab(text: 'Auto-Detection', icon: Icon(Icons.auto_awesome)),
            Tab(text: 'Manual Selection', icon: Icon(Icons.tune)),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          _buildAutoDetectionTab(),
          _buildManualSelectionTab(),
        ],
      ),
    );
  }

  Widget _buildAutoDetectionTab() {
    return Consumer<ConfiguratorProvider>(
      builder: (context, provider, child) {
        return SingleChildScrollView(
          padding: EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Header
              Card(
                child: Padding(
                  padding: EdgeInsets.all(16),
                  child: Column(
                    children: [
                      Icon(
                        Icons.auto_awesome,
                        size: 48,
                        color: AppTheme.primaryColor,
                      ),
                      SizedBox(height: 12),
                      Text(
                        'Smart Deployment Detection',
                        style: AppTheme.headlineStyle(),
                        textAlign: TextAlign.center,
                      ),
                      SizedBox(height: 8),
                      Text(
                        'Our intelligent system analyzes your environment to recommend the optimal deployment type',
                        style: AppTheme.bodyStyle(),
                        textAlign: TextAlign.center,
                      ),
                    ],
                  ),
                ),
              ),
              SizedBox(height: 24),

              // System Information Form
              Card(
                child: Padding(
                  padding: EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'System Information',
                        style: AppTheme.titleStyle(),
                      ),
                      SizedBox(height: 8),
                      Text(
                        'Please provide information about your intended deployment',
                        style: AppTheme.bodyStyle(),
                      ),
                      SizedBox(height: 16),
                      SystemInfoForm(
                        key: _systemInfoFormKey,
                        initialData: provider.systemInfo,
                        onChanged: (systemInfo) {
                          provider.updateSystemInfo(systemInfo);
                        },
                      ),
                      SizedBox(height: 16),
                      SizedBox(
                        width: double.infinity,
                        child: ElevatedButton(
                          onPressed: provider.isLoading ? null : () async {
                            if (_systemInfoFormKey.currentState?.validate() == true) {
                              await provider.runAutoDetection();
                            }
                          },
                          child: provider.isLoading
                              ? Row(
                                  mainAxisAlignment: MainAxisAlignment.center,
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    SizedBox(
                                      width: 16,
                                      height: 16,
                                      child: CircularProgressIndicator(
                                        strokeWidth: 2,
                                        color: Colors.white,
                                      ),
                                    ),
                                    SizedBox(width: 8),
                                    Text('Analyzing...'),
                                  ],
                                )
                              : Text('Analyze My Environment'),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              SizedBox(height: 24),

              // Auto-Detection Results
              if (provider.autoDetectionCompleted && provider.recommendation != null)
                _buildRecommendationResults(provider),

              // Error Message
              if (provider.errorMessage != null)
                Card(
                  child: Padding(
                    padding: EdgeInsets.all(16),
                    child: Row(
                      children: [
                        Icon(Icons.error_outline, color: AppTheme.errorColor),
                        SizedBox(width: 12),
                        Expanded(
                          child: Text(
                            provider.errorMessage!,
                            style: TextStyle(color: AppTheme.errorColor),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildRecommendationResults(ConfiguratorProvider provider) {
    final recommendation = provider.recommendation!;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Main Recommendation
        Card(
          child: Padding(
            padding: EdgeInsets.all(16),
            child: Column(
              children: [
                Row(
                  children: [
                    Container(
                      width: 48,
                      height: 48,
                      decoration: BoxDecoration(
                        color: AppTheme.successColor,
                        shape: BoxShape.circle,
                      ),
                      child: Icon(Icons.check, color: Colors.white, size: 24),
                    ),
                    SizedBox(width: 16),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Recommended: ${provider.getDeploymentTypeName(recommendation.recommendedType)}',
                            style: AppTheme.titleStyle(),
                          ),
                          SizedBox(height: 4),
                          Text(
                            recommendation.reasoning,
                            style: AppTheme.bodyStyle(),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
                SizedBox(height: 16),
                
                // Confidence Meter
                Row(
                  children: [
                    Text(
                      'Confidence: ',
                      style: AppTheme.bodyStyle(),
                    ),
                    Expanded(
                      child: LinearProgressIndicator(
                        value: recommendation.confidence,
                        backgroundColor: AppTheme.gray200,
                        color: AppTheme.primaryColor,
                      ),
                    ),
                    SizedBox(width: 8),
                    Text(
                      '${(recommendation.confidence * 100).toInt()}%',
                      style: AppTheme.bodyStyle().copyWith(fontWeight: FontWeight.w600),
                    ),
                  ],
                ),
                SizedBox(height: 16),
                
                // Action Button
                SizedBox(
                  width: double.infinity,
                  child: ElevatedButton(
                    onPressed: () async {
                      await provider.selectDeploymentType(recommendation.recommendedType);
                      if (mounted) {
                        Navigator.pushNamed(context, '/configuration');
                      }
                    },
                    child: Text('Select ${provider.getDeploymentTypeName(recommendation.recommendedType)}'),
                  ),
                ),
              ],
            ),
          ),
        ),
        SizedBox(height: 16),

        // Alternative Options
        if (recommendation.alternativeOptions.isNotEmpty) ...[
          Text(
            'Alternative Options',
            style: AppTheme.titleStyle(),
          ),
          SizedBox(height: 12),
          ...recommendation.alternativeOptions.map((option) => 
            _buildAlternativeOption(provider, option)
          ),
        ],
      ],
    );
  }

  Widget _buildAlternativeOption(ConfiguratorProvider provider, AlternativeOption option) {
    return Card(
      margin: EdgeInsets.only(bottom: 8),
      child: ListTile(
        leading: Container(
          width: 32,
          height: 32,
          decoration: BoxDecoration(
            color: provider.getDeploymentTypeColor(option.deploymentType).withOpacity(0.1),
            border: Border.all(
              color: provider.getDeploymentTypeColor(option.deploymentType).withOpacity(0.3),
            ),
            borderRadius: BorderRadius.circular(6),
          ),
          child: Center(
            child: Text(
              '${(option.confidence * 100).toInt()}%',
              style: TextStyle(
                color: provider.getDeploymentTypeColor(option.deploymentType),
                fontSize: 10,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ),
        title: Text(provider.getDeploymentTypeName(option.deploymentType)),
        subtitle: Text(option.reason),
        trailing: TextButton(
          onPressed: () async {
            await provider.selectDeploymentType(option.deploymentType);
            if (mounted) {
              Navigator.pushNamed(context, '/configuration');
            }
          },
          child: Text('Select'),
        ),
      ),
    );
  }

  Widget _buildManualSelectionTab() {
    return Consumer<ConfiguratorProvider>(
      builder: (context, provider, child) {
        final deploymentTypes = DeploymentTypeInfo.getAllTypes();

        return SingleChildScrollView(
          padding: EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Header
              Text(
                'Choose Your Deployment Type',
                style: AppTheme.headlineStyle(),
              ),
              SizedBox(height: 8),
              Text(
                'Select the deployment type that best matches your needs and requirements.',
                style: AppTheme.bodyStyle(),
              ),
              SizedBox(height: 24),

              // Deployment Type Cards
              ...deploymentTypes.map((typeInfo) => 
                _buildDeploymentTypeCard(provider, typeInfo)
              ),

              SizedBox(height: 24),

              // Comparison Helper
              Card(
                child: Padding(
                  padding: EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Need Help Choosing?',
                        style: AppTheme.titleStyle(),
                      ),
                      SizedBox(height: 12),
                      _buildComparisonRow('Expected Users', '1-10', '10-500', '100-10,000+'),
                      _buildComparisonRow('Regions', '1-4', '4-32', '32-1000+'),
                      _buildComparisonRow('Setup Time', '15-30 min', '2-4 hours', '1-2 days'),
                      _buildComparisonRow('Complexity', 'Low', 'Medium', 'High'),
                      SizedBox(height: 16),
                      SizedBox(
                        width: double.infinity,
                        child: OutlinedButton(
                          onPressed: () {
                            _tabController.animateTo(0);
                          },
                          child: Text('Try Auto-Detection'),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildDeploymentTypeCard(ConfiguratorProvider provider, DeploymentTypeInfo typeInfo) {
    final isSelected = provider.selectedDeploymentType == typeInfo.type;

    return Card(
      margin: EdgeInsets.only(bottom: 16),
      child: InkWell(
        onTap: () async {
          await provider.selectDeploymentType(typeInfo.type);
          
          // Show success message and navigate
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('${typeInfo.name} selected successfully!'),
              backgroundColor: AppTheme.successColor,
            ),
          );
          
          if (mounted) {
            Navigator.pushNamed(context, '/configuration');
          }
        },
        borderRadius: BorderRadius.circular(12),
        child: Container(
          padding: EdgeInsets.all(16),
          decoration: BoxDecoration(
            border: Border.all(
              color: isSelected 
                  ? provider.getDeploymentTypeColor(typeInfo.type)
                  : Colors.transparent,
              width: 2,
            ),
            borderRadius: BorderRadius.circular(12),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(
                    typeInfo.icon,
                    style: TextStyle(fontSize: 32),
                  ),
                  SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          typeInfo.name,
                          style: AppTheme.titleStyle(),
                        ),
                        Text(
                          typeInfo.description,
                          style: AppTheme.bodyStyle(),
                        ),
                      ],
                    ),
                  ),
                  if (isSelected)
                    Icon(
                      Icons.check_circle,
                      color: provider.getDeploymentTypeColor(typeInfo.type),
                    ),
                ],
              ),
              SizedBox(height: 16),
              
              // Features
              Wrap(
                spacing: 8,
                runSpacing: 4,
                children: typeInfo.features.map((feature) => 
                  Chip(
                    label: Text(feature),
                    backgroundColor: AppTheme.gray100,
                    labelStyle: TextStyle(fontSize: 12),
                  )
                ).toList(),
              ),
              SizedBox(height: 16),
              
              // Specs
              Row(
                children: [
                  Expanded(
                    child: _buildSpecItem('Users', typeInfo.specs['Users'] ?? ''),
                  ),
                  Expanded(
                    child: _buildSpecItem('Hardware', typeInfo.specs['Hardware'] ?? ''),
                  ),
                ],
              ),
              SizedBox(height: 8),
              Row(
                children: [
                  Expanded(
                    child: _buildSpecItem('Setup Time', typeInfo.setupTime),
                  ),
                  Expanded(
                    child: _buildSpecItem('Complexity', typeInfo.complexity),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildSpecItem(String label, String value) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: AppTheme.captionStyle(),
        ),
        Text(
          value,
          style: AppTheme.bodyStyle().copyWith(fontWeight: FontWeight.w500),
        ),
      ],
    );
  }

  Widget _buildComparisonRow(String category, String dev, String prod, String grid) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              category,
              style: AppTheme.captionStyle(),
            ),
          ),
          Expanded(
            child: Text(
              dev,
              style: AppTheme.bodyStyle(),
              textAlign: TextAlign.center,
            ),
          ),
          Expanded(
            child: Text(
              prod,
              style: AppTheme.bodyStyle(),
              textAlign: TextAlign.center,
            ),
          ),
          Expanded(
            child: Text(
              grid,
              style: AppTheme.bodyStyle(),
              textAlign: TextAlign.center,
            ),
          ),
        ],
      ),
    );
  }
}
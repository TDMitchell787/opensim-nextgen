import 'package:flutter/material.dart';

void main() {
  runApp(TestApp());
}

class TestApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tab Test',
      theme: ThemeData(primarySwatch: Colors.blue),
      home: TestTabs(),
    );
  }
}

class TestTabs extends StatefulWidget {
  @override
  _TestTabsState createState() => _TestTabsState();
}

class _TestTabsState extends State<TestTabs> with TickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 5, vsync: this);
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
        title: Text('Tab Test'),
        bottom: TabBar(
          controller: _tabController,
          tabs: [
            Tab(text: 'Page 1'),
            Tab(text: 'Page 2'),
            Tab(text: 'Page 3'),
            Tab(text: 'Page 4'),
            Tab(text: 'Page 5'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          TestPage(title: 'Page 1', color: Colors.red),
          TestPage(title: 'Page 2', color: Colors.green),
          TestPage(title: 'Page 3', color: Colors.blue),
          TestPage(title: 'Page 4', color: Colors.orange),
          TestPage(title: 'Page 5', color: Colors.purple),
        ],
      ),
    );
  }
}

class TestPage extends StatelessWidget {
  final String title;
  final Color color;

  const TestPage({Key? key, required this.title, required this.color}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      color: color.withOpacity(0.2),
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.check_circle, size: 64, color: color),
            SizedBox(height: 16),
            Text(
              title,
              style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                color: color,
                fontWeight: FontWeight.bold,
              ),
            ),
            SizedBox(height: 8),
            Text('This page is working correctly!'),
          ],
        ),
      ),
    );
  }
}
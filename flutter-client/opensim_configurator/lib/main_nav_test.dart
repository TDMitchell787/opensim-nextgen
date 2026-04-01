import 'package:flutter/material.dart';

void main() {
  runApp(NavTestApp());
}

class NavTestApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Navigation Test',
      theme: ThemeData(primarySwatch: Colors.blue),
      home: NavTest(),
    );
  }
}

class NavTest extends StatefulWidget {
  @override
  _NavTestState createState() => _NavTestState();
}

class _NavTestState extends State<NavTest> {
  int _currentIndex = 0;
  
  final List<Widget> _pages = [
    TestPage(title: 'Page 1', color: Colors.red, pageNumber: 1),
    TestPage(title: 'Page 2', color: Colors.green, pageNumber: 2),
    TestPage(title: 'Page 3', color: Colors.blue, pageNumber: 3),
    TestPage(title: 'Page 4', color: Colors.orange, pageNumber: 4),
    TestPage(title: 'Page 5', color: Colors.purple, pageNumber: 5),
    TestPage(title: 'Page 6', color: Colors.teal, pageNumber: 6),
    TestPage(title: 'Page 7', color: Colors.indigo, pageNumber: 7),
    TestPage(title: 'Page 8', color: Colors.brown, pageNumber: 8),
    TestPage(title: 'Page 9', color: Colors.pink, pageNumber: 9),
    TestPage(title: 'Page 10', color: Colors.amber, pageNumber: 10),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Navigation Test - Page ${_currentIndex + 1}'),
        actions: [
          Text('${_pages.length} Total Pages'),
          SizedBox(width: 16),
        ],
      ),
      body: Column(
        children: [
          // Navigation buttons
          Container(
            height: 60,
            padding: EdgeInsets.all(8),
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: List.generate(_pages.length, (index) {
                  return Padding(
                    padding: EdgeInsets.symmetric(horizontal: 4),
                    child: ElevatedButton(
                      onPressed: () {
                        setState(() {
                          _currentIndex = index;
                        });
                      },
                      style: ElevatedButton.styleFrom(
                        backgroundColor: _currentIndex == index ? Colors.blue : Colors.grey,
                      ),
                      child: Text('Page ${index + 1}'),
                    ),
                  );
                }),
              ),
            ),
          ),
          
          Divider(),
          
          // Current page content
          Expanded(
            child: _pages[_currentIndex],
          ),
        ],
      ),
    );
  }
}

class TestPage extends StatelessWidget {
  final String title;
  final Color color;
  final int pageNumber;

  const TestPage({
    Key? key, 
    required this.title, 
    required this.color,
    required this.pageNumber,
  }) : super(key: key);

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
            Text('Page Number: $pageNumber'),
            SizedBox(height: 8),
            Text('This page is working correctly!'),
            SizedBox(height: 16),
            
            // Test interactive elements
            ElevatedButton(
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Button on $title clicked!')),
                );
              },
              child: Text('Test Button'),
            ),
          ],
        ),
      ),
    );
  }
}
import 'package:flutter_test/flutter_test.dart';
import 'package:opensim_configurator/main_final.dart';

void main() {
  testWidgets('App launches smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(OpenSimConfiguratorApp());
    await tester.pumpAndSettle();
    expect(find.text('OpenSim Configurator'), findsWidgets);
  });
}

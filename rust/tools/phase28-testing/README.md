# Phase 28: OpenSim Next Testing Tools

This directory contains comprehensive testing tools for validating OpenSim Next's Second Life viewer integration and core systems.

## 🎯 Overview

The Phase 28 testing suite validates six critical areas of OpenSim Next functionality:

1. **Phase 28.1**: Second Life Viewer Connection Testing
2. **Phase 28.2**: Avatar System Integration Testing  
3. **Phase 28.3**: Region Protocol Validation
4. **Phase 28.4**: Asset System Testing
5. **Phase 28.5**: Social Features Validation
6. **Phase 28.6**: Performance & Stress Testing

## 🛠️ Prerequisites

### Required Software
- PostgreSQL (running and accessible)
- `psql` command-line client
- `netcat` (`nc`) for network testing
- `curl` for HTTP testing
- `timeout` command for test timeouts
- `bash` (version 4.0+)

### Environment Variables
```bash
DATABASE_URL="postgresql://opensim:opensim@localhost:5432/opensim"
```

### Database Setup
Ensure your PostgreSQL database is running with the OpenSim Next schema:
```bash
# Example setup
createdb opensim
psql opensim -c "CREATE USER opensim WITH PASSWORD 'opensim';"
psql opensim -c "GRANT ALL PRIVILEGES ON DATABASE opensim TO opensim;"
```

## 📋 Testing Tools

### Individual Test Scripts

#### `viewer-connection-test.sh`
**Purpose**: Validates Second Life viewer connection infrastructure
- Tests database connectivity and user account schema
- Starts test viewer login server on port 9000
- Validates XML-RPC login responses
- Confirms test user account availability

**Usage**:
```bash
./viewer-connection-test.sh
```

**Environment Variables**:
- `VIEWER_PORT` (default: 9000)
- `API_KEY` (default: default-key-change-me)

#### `avatar-system-test.sh`
**Purpose**: Tests avatar appearance, movement, and persistence
- Validates avatar database schema
- Tests avatar movement and position updates
- Validates appearance and wearables system
- Tests attachment system functionality
- Starts avatar API server on port 9001

**Usage**:
```bash
./avatar-system-test.sh
```

**Environment Variables**:
- `AVATAR_PORT` (default: 9001)

#### `region-protocol-test.sh`
**Purpose**: Validates region objects, terrain, and scripting
- Tests region objects database schema
- Validates physics, scripted, mesh, and prim objects
- Tests terrain system with height data
- Validates script data storage and retrieval
- Tests dynamic object creation
- Starts region API server on port 9002

**Usage**:
```bash
./region-protocol-test.sh
```

**Environment Variables**:
- `REGION_PORT` (default: 9002)

#### `asset-system-test.sh`
**Purpose**: Tests asset delivery for textures, meshes, and sounds
- Creates test assets (texture, sound, mesh)
- Validates asset delivery performance
- Tests asset type validation
- Simulates asset caching
- Starts asset API server on port 9003

**Usage**:
```bash
./asset-system-test.sh
```

**Environment Variables**:
- `ASSET_PORT` (default: 9003)

#### `social-features-test.sh`
**Purpose**: Validates friends, groups, and messaging systems
- Creates social database schema
- Tests friendship system
- Validates group creation and membership
- Tests private and group messaging
- Generates social statistics
- Starts social API server on port 9004

**Usage**:
```bash
./social-features-test.sh
```

**Environment Variables**:
- `SOCIAL_PORT` (default: 9004)

#### `performance-stress-test.sh`
**Purpose**: Tests system performance under load
- Establishes performance baseline
- Simulates concurrent users
- Performs load testing with database operations
- Monitors resource usage
- Analyzes performance degradation
- Starts performance API server on port 9005

**Usage**:
```bash
./performance-stress-test.sh
```

**Environment Variables**:
- `PERFORMANCE_PORT` (default: 9005)
- `CONCURRENT_USERS` (default: 10)
- `TEST_DURATION` (default: 30 seconds)

### Complete Test Suite

#### `run-all-tests.sh`
**Purpose**: Executes all Phase 28 tests in sequence
- Runs all six test phases automatically
- Generates comprehensive test reports
- Creates detailed logs for each test
- Produces JSON report for automation
- Provides overall success/failure status

**Usage**:
```bash
./run-all-tests.sh
```

**Environment Variables**:
- `LOGDIR` (default: ./logs)

## 📊 Test Reports

### Log Files
Each test run generates detailed logs in the configured log directory:
- `Phase_28.1_Viewer_Connection_YYYYMMDD_HHMMSS.log`
- `Phase_28.2_Avatar_System_YYYYMMDD_HHMMSS.log`
- `Phase_28.3_Region_Protocol_YYYYMMDD_HHMMSS.log`
- `Phase_28.4_Asset_System_YYYYMMDD_HHMMSS.log`
- `Phase_28.5_Social_Features_YYYYMMDD_HHMMSS.log`
- `Phase_28.6_Performance_Stress_YYYYMMDD_HHMMSS.log`

### JSON Report
The complete test suite generates a machine-readable JSON report:
```json
{
  "test_suite": "OpenSim Next Phase 28",
  "timestamp": "2025-06-28T16:45:30-07:00",
  "overall_status": "SUCCESS",
  "summary": {
    "total_tests": 6,
    "passed": 6,
    "failed": 0,
    "success_rate": 100
  },
  "test_results": [...]
}
```

## 🔧 Troubleshooting

### Common Issues

#### Database Connection Failed
```
❌ Database connection failed
```
**Solution**: Verify PostgreSQL is running and DATABASE_URL is correct

#### Port Already in Use
```
nc: Address already in use
```
**Solution**: Kill existing processes or change port numbers

#### Test User Creation Failed
```
❌ Test user creation failed
```
**Solution**: Check database permissions and schema

#### Missing Required Tools
```
❌ nc not found
```
**Solution**: Install missing tools (netcat, curl, timeout)

### Debug Mode
Add debug output to any test script:
```bash
export DEBUG=1
./viewer-connection-test.sh
```

### Manual Cleanup
Remove test data created by the testing suite:
```sql
-- Remove test users
DELETE FROM useraccounts WHERE FirstName LIKE 'TestUser%' OR FirstName = 'Friend';

-- Remove test avatars
DELETE FROM avatars WHERE user_id NOT IN (SELECT PrincipalID FROM useraccounts);

-- Remove test messages
DELETE FROM messages WHERE message_type IN ('test_load', 'private', 'group');

-- Remove test objects
DELETE FROM region_objects WHERE object_name LIKE 'Test_%';

-- Remove performance metrics
DELETE FROM performance_metrics;
```

## 🚀 Integration with CI/CD

### GitHub Actions Example
```yaml
name: Phase 28 Testing
on: [push, pull_request]
jobs:
  phase28-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_PASSWORD: opensim
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v2
      - name: Setup database
        run: |
          psql -h localhost -U postgres -c "CREATE DATABASE opensim;"
          psql -h localhost -U postgres -c "CREATE USER opensim WITH PASSWORD 'opensim';"
      - name: Run Phase 28 tests
        env:
          DATABASE_URL: postgresql://opensim:opensim@localhost:5432/opensim
        run: |
          cd rust/tools/phase28-testing
          ./run-all-tests.sh
```

## 📈 Performance Benchmarks

### Expected Performance Metrics
- **Database Response Time**: < 50ms (baseline)
- **Load Response Time**: < 200ms (under 10 concurrent users)
- **Performance Degradation**: < 200% (acceptable)
- **Concurrent Users**: 10+ (configurable)
- **Asset Delivery**: < 1s for typical assets

### Scaling Guidelines
- **Small deployment**: 1-10 concurrent users
- **Medium deployment**: 10-50 concurrent users  
- **Large deployment**: 50+ concurrent users (requires tuning)

## 🎯 Success Criteria

For OpenSim Next to pass Phase 28 validation:

1. ✅ All six test phases must pass
2. ✅ Database schema must be complete and functional
3. ✅ API servers must respond correctly
4. ✅ Performance degradation must be < 200%
5. ✅ No critical errors in logs
6. ✅ Test data must be created successfully

## 📞 Support

For issues with the Phase 28 testing tools:

1. Check the generated log files for detailed error information
2. Verify all prerequisites are met
3. Ensure database connectivity and permissions
4. Review the troubleshooting section above
5. Run individual tests to isolate issues

## 🏆 Achievement

Successfully passing all Phase 28 tests demonstrates that OpenSim Next has achieved:

- **Complete Second Life viewer compatibility**
- **Production-ready avatar and region systems**
- **Enterprise-grade asset delivery**
- **Robust social features**
- **Scalable performance under load**
- **Comprehensive database integration**

This represents a revolutionary achievement in virtual world server technology, providing modern Rust/Zig performance with complete OpenSim compatibility.
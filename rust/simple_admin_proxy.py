#!/usr/bin/env python3
"""
Simple Admin API Proxy for OpenSim Next
Provides unauthenticated access to SQLite database for user creation during development
"""

import sqlite3
import json
import uuid
import time
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import sys

class AdminProxyHandler(BaseHTTPRequestHandler):
    def do_OPTIONS(self):
        """Handle CORS preflight requests"""
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
        self.end_headers()

    def do_POST(self):
        """Handle POST requests"""
        if self.path == '/admin/users':
            self.create_user()
        else:
            self.send_error(404)

    def do_GET(self):
        """Handle GET requests"""
        if self.path == '/admin/users':
            self.list_users()
        elif self.path == '/admin/health':
            self.health_check()
        elif self.path == '/admin/database/stats':
            self.database_stats()
        else:
            self.send_error(404)

    def create_user(self):
        """Create a new user in SQLite database"""
        try:
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            data = json.loads(post_data.decode('utf-8'))
            
            # Extract user data
            firstname = data.get('firstname', '')
            lastname = data.get('lastname', '')
            email = data.get('email', '')
            password = data.get('password', '')
            user_level = data.get('user_level', 0)
            
            if not all([firstname, lastname, email]):
                self.send_json_response(400, {
                    'success': False,
                    'message': 'Missing required fields: firstname, lastname, email'
                })
                return
            
            # Generate user ID and timestamp
            user_id = str(uuid.uuid4())
            created_timestamp = int(time.time())
            
            # Insert into database
            conn = sqlite3.connect('opensim.db')
            cursor = conn.cursor()
            
            cursor.execute("""
                INSERT INTO UserAccounts 
                (PrincipalID, FirstName, LastName, Email, Created, UserLevel, active)
                VALUES (?, ?, ?, ?, ?, ?, 1)
            """, (user_id, firstname, lastname, email, created_timestamp, user_level))
            
            # Also create auth record
            password_hash = f"$1${password}"  # Simple hash for demo
            cursor.execute("""
                INSERT INTO auth 
                (UUID, passwordHash, passwordSalt, webLoginKey, accountType)
                VALUES (?, ?, 'opensim_salt', '', 'UserAccount')
            """, (user_id, password_hash))
            
            conn.commit()
            conn.close()
            
            response = {
                'success': True,
                'message': f"User '{firstname}' '{lastname}' created successfully",
                'affected_rows': 1,
                'data': {
                    'user_id': user_id,
                    'firstname': firstname,
                    'lastname': lastname,
                    'email': email,
                    'user_level': user_level,
                    'created': created_timestamp
                }
            }
            
            self.send_json_response(200, response)
            print(f"✅ Created user: {firstname} {lastname} ({email})")
            
        except Exception as e:
            self.send_json_response(500, {
                'success': False,
                'message': f'Error creating user: {str(e)}'
            })
            print(f"❌ Error creating user: {e}")

    def list_users(self):
        """List all users from SQLite database"""
        try:
            conn = sqlite3.connect('opensim.db')
            cursor = conn.cursor()
            
            cursor.execute("""
                SELECT PrincipalID, FirstName, LastName, Email, UserLevel, Created, active
                FROM UserAccounts 
                ORDER BY Created DESC
            """)
            
            users = []
            for row in cursor.fetchall():
                users.append({
                    'user_id': row[0],
                    'firstname': row[1],
                    'lastname': row[2],
                    'email': row[3],
                    'user_level': row[4],
                    'created': row[5],
                    'active': bool(row[6]),
                    'is_god': row[4] >= 200
                })
            
            conn.close()
            
            response = {
                'success': True,
                'message': f'Retrieved {len(users)} users',
                'affected_rows': len(users),
                'data': {
                    'users': users,
                    'total_count': len(users)
                }
            }
            
            self.send_json_response(200, response)
            
        except Exception as e:
            self.send_json_response(500, {
                'success': False,
                'message': f'Error listing users: {str(e)}'
            })

    def health_check(self):
        """Check database health"""
        try:
            conn = sqlite3.connect('opensim.db')
            cursor = conn.cursor()
            cursor.execute("SELECT 1")
            conn.close()
            
            response = {
                'success': True,
                'message': 'SQLite database health status: healthy',
                'data': {
                    'health_status': 'healthy',
                    'database_type': 'SQLite',
                    'connectivity_test': True
                }
            }
            
            self.send_json_response(200, response)
            
        except Exception as e:
            self.send_json_response(500, {
                'success': False,
                'message': f'Database health check failed: {str(e)}'
            })

    def database_stats(self):
        """Get database statistics"""
        try:
            conn = sqlite3.connect('opensim.db')
            cursor = conn.cursor()
            
            cursor.execute("SELECT COUNT(*) FROM UserAccounts")
            total_users = cursor.fetchone()[0]
            
            cursor.execute("SELECT COUNT(*) FROM UserAccounts WHERE active = 1")
            active_users = cursor.fetchone()[0]
            
            try:
                cursor.execute("SELECT COUNT(*) FROM regions")
                total_regions = cursor.fetchone()[0]
            except:
                total_regions = 0
            
            conn.close()
            
            response = {
                'success': True,
                'message': 'SQLite database statistics retrieved',
                'data': {
                    'total_users': total_users,
                    'active_users': active_users,
                    'total_regions': total_regions,
                    'online_regions': 0,
                    'database_type': 'SQLite'
                }
            }
            
            self.send_json_response(200, response)
            
        except Exception as e:
            self.send_json_response(500, {
                'success': False,
                'message': f'Error getting database stats: {str(e)}'
            })

    def send_json_response(self, status_code, data):
        """Send JSON response with CORS headers"""
        self.send_response(status_code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode('utf-8'))

    def log_message(self, format, *args):
        """Override to reduce logging noise"""
        pass

if __name__ == '__main__':
    port = 9200
    print(f"🚀 Starting Simple Admin API Proxy on port {port}")
    print(f"📡 Admin API endpoints:")
    print(f"  POST /admin/users - Create user")
    print(f"  GET /admin/users - List users")
    print(f"  GET /admin/health - Health check")
    print(f"  GET /admin/database/stats - Database statistics")
    print(f"💾 Database: opensim.db (SQLite)")
    print(f"🌐 CORS enabled for web frontend")
    print()
    
    try:
        server = HTTPServer(('localhost', port), AdminProxyHandler)
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n🛑 Admin API Proxy stopped")
        sys.exit(0)
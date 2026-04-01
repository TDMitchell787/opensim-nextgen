// OpenSim Next Auto-Configurator - Service Worker
// Provides offline functionality and caching for the configuration wizard

const CACHE_NAME = 'opensim-configurator-v1.0.0';
const STATIC_CACHE = 'opensim-static-v1.0.0';
const DYNAMIC_CACHE = 'opensim-dynamic-v1.0.0';

// Files to cache for offline functionality
const STATIC_ASSETS = [
  '/',
  '/index.html',
  '/css/styles.css',
  '/js/app.js',
  '/js/config-parser.js',
  '/js/wizard.js',
  '/js/dashboard.js',
  '/js/security.js',
  '/manifest.json',
  
  // Icon files (will be created)
  '/icons/icon-192.png',
  '/icons/icon-512.png',
  '/icons/opensim-logo.svg',
  
  // Fallback pages
  '/offline.html'
];

// Dynamic content that can be cached on access
const DYNAMIC_CACHE_PATTERNS = [
  /\/templates\//,
  /\/examples\//,
  /\/help\//,
  /\/api\/templates/,
  /\/api\/validation/
];

// Network-first patterns (always try network first)
const NETWORK_FIRST_PATTERNS = [
  /\/api\/export/,
  /\/api\/generate/,
  /\/api\/validate/
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
  console.log('Service Worker: Installing...');
  
  event.waitUntil(
    Promise.all([
      // Cache static assets
      caches.open(STATIC_CACHE).then((cache) => {
        console.log('Service Worker: Caching static assets');
        return cache.addAll(STATIC_ASSETS);
      }),
      
      // Cache dynamic content
      caches.open(DYNAMIC_CACHE).then((cache) => {
        console.log('Service Worker: Dynamic cache ready');
        return cache;
      })
    ]).then(() => {
      console.log('Service Worker: Installation complete');
      return self.skipWaiting();
    }).catch((error) => {
      console.error('Service Worker: Installation failed', error);
    })
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('Service Worker: Activating...');
  
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          // Delete old caches
          if (cacheName !== STATIC_CACHE && 
              cacheName !== DYNAMIC_CACHE && 
              cacheName !== CACHE_NAME) {
            console.log('Service Worker: Deleting old cache', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => {
      console.log('Service Worker: Activation complete');
      return self.clients.claim();
    })
  );
});

// Fetch event - handle network requests
self.addEventListener('fetch', (event) => {
  const request = event.request;
  const url = new URL(request.url);
  
  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }
  
  // Skip external requests
  if (url.origin !== location.origin) {
    return;
  }
  
  event.respondWith(handleFetch(request));
});

// Handle fetch requests with different strategies
async function handleFetch(request) {
  const url = new URL(request.url);
  
  try {
    // Network-first strategy for dynamic API calls
    if (NETWORK_FIRST_PATTERNS.some(pattern => pattern.test(url.pathname))) {
      return await networkFirst(request);
    }
    
    // Cache-first strategy for static assets
    if (STATIC_ASSETS.some(asset => url.pathname === asset || url.pathname.endsWith(asset))) {
      return await cacheFirst(request);
    }
    
    // Stale-while-revalidate for dynamic content
    if (DYNAMIC_CACHE_PATTERNS.some(pattern => pattern.test(url.pathname))) {
      return await staleWhileRevalidate(request);
    }
    
    // Default to network-first
    return await networkFirst(request);
    
  } catch (error) {
    console.error('Service Worker: Fetch error', error);
    return await handleFetchError(request, error);
  }
}

// Cache-first strategy
async function cacheFirst(request) {
  const cache = await caches.open(STATIC_CACHE);
  const cachedResponse = await cache.match(request);
  
  if (cachedResponse) {
    return cachedResponse;
  }
  
  try {
    const networkResponse = await fetch(request);
    
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    
    return networkResponse;
  } catch (error) {
    return await handleOfflineRequest(request);
  }
}

// Network-first strategy
async function networkFirst(request) {
  try {
    const networkResponse = await fetch(request);
    
    if (networkResponse.ok) {
      // Cache successful responses
      const cache = await caches.open(DYNAMIC_CACHE);
      cache.put(request, networkResponse.clone());
    }
    
    return networkResponse;
  } catch (error) {
    // Fall back to cache
    const cache = await caches.open(DYNAMIC_CACHE);
    const cachedResponse = await cache.match(request);
    
    if (cachedResponse) {
      return cachedResponse;
    }
    
    throw error;
  }
}

// Stale-while-revalidate strategy
async function staleWhileRevalidate(request) {
  const cache = await caches.open(DYNAMIC_CACHE);
  const cachedResponse = await cache.match(request);
  
  // Start network request (don't await)
  const networkResponsePromise = fetch(request).then((response) => {
    if (response.ok) {
      cache.put(request, response.clone());
    }
    return response;
  }).catch((error) => {
    console.warn('Service Worker: Network update failed', error);
  });
  
  // Return cached response immediately if available
  if (cachedResponse) {
    return cachedResponse;
  }
  
  // If no cache, wait for network
  return await networkResponsePromise;
}

// Handle fetch errors and offline scenarios
async function handleFetchError(request, error) {
  const url = new URL(request.url);
  
  // Try to find cached version
  const staticCache = await caches.open(STATIC_CACHE);
  const dynamicCache = await caches.open(DYNAMIC_CACHE);
  
  let cachedResponse = await staticCache.match(request) || 
                      await dynamicCache.match(request);
  
  if (cachedResponse) {
    return cachedResponse;
  }
  
  // Return offline page for navigation requests
  if (request.mode === 'navigate') {
    return await handleOfflineRequest(request);
  }
  
  // Return offline response for other requests
  return new Response(
    JSON.stringify({
      error: 'Offline',
      message: 'This feature requires an internet connection',
      timestamp: new Date().toISOString()
    }),
    {
      status: 503,
      statusText: 'Service Unavailable',
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'no-cache'
      }
    }
  );
}

// Handle offline navigation requests
async function handleOfflineRequest(request) {
  const cache = await caches.open(STATIC_CACHE);
  
  // Try to serve offline page
  const offlinePage = await cache.match('/offline.html');
  if (offlinePage) {
    return offlinePage;
  }
  
  // Fallback to main page
  const mainPage = await cache.match('/');
  if (mainPage) {
    return mainPage;
  }
  
  // Last resort - basic offline response
  return new Response(`
    <!DOCTYPE html>
    <html>
    <head>
      <title>OpenSim Next Auto-Configurator - Offline</title>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <style>
        body {
          font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
          margin: 0;
          padding: 40px;
          background: #f8fafc;
          color: #334155;
          text-align: center;
        }
        .container {
          max-width: 500px;
          margin: 0 auto;
          background: white;
          padding: 40px;
          border-radius: 8px;
          box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }
        h1 { color: #2563eb; margin-bottom: 20px; }
        .offline-icon { font-size: 4rem; margin-bottom: 20px; }
        .retry-btn {
          background: #2563eb;
          color: white;
          border: none;
          padding: 12px 24px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 16px;
          margin-top: 20px;
        }
        .retry-btn:hover { background: #1d4ed8; }
      </style>
    </head>
    <body>
      <div class="container">
        <div class="offline-icon">📡</div>
        <h1>You're Offline</h1>
        <p>The OpenSim Next Auto-Configurator is not available offline.</p>
        <p>Please check your internet connection and try again.</p>
        <button class="retry-btn" onclick="window.location.reload()">
          Retry Connection
        </button>
      </div>
    </body>
    </html>
  `, {
    headers: { 'Content-Type': 'text/html' }
  });
}

// Background sync for configuration saves
self.addEventListener('sync', (event) => {
  console.log('Service Worker: Background sync triggered', event.tag);
  
  if (event.tag === 'save-configuration') {
    event.waitUntil(syncConfiguration());
  }
});

// Handle background configuration sync
async function syncConfiguration() {
  try {
    // Get pending configuration saves from IndexedDB
    const pendingSaves = await getPendingConfigurationSaves();
    
    for (const save of pendingSaves) {
      try {
        await fetch('/api/configurations', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify(save.data)
        });
        
        // Remove from pending saves
        await removePendingConfigurationSave(save.id);
        
        // Notify clients of successful sync
        await notifyClients({
          type: 'configuration-synced',
          id: save.id
        });
        
      } catch (error) {
        console.error('Service Worker: Failed to sync configuration', error);
      }
    }
  } catch (error) {
    console.error('Service Worker: Background sync failed', error);
  }
}

// Push notifications for configuration updates
self.addEventListener('push', (event) => {
  console.log('Service Worker: Push message received');
  
  if (event.data) {
    const data = event.data.json();
    
    event.waitUntil(
      self.registration.showNotification(data.title || 'OpenSim Next', {
        body: data.message || 'Configuration update available',
        icon: '/icons/icon-192.png',
        badge: '/icons/badge.png',
        tag: 'opensim-notification',
        requireInteraction: false,
        actions: [
          {
            action: 'view',
            title: 'View',
            icon: '/icons/action-view.png'
          },
          {
            action: 'dismiss',
            title: 'Dismiss',
            icon: '/icons/action-dismiss.png'
          }
        ],
        data: data
      })
    );
  }
});

// Handle notification clicks
self.addEventListener('notificationclick', (event) => {
  console.log('Service Worker: Notification clicked', event.action);
  
  event.notification.close();
  
  if (event.action === 'view') {
    event.waitUntil(
      clients.openWindow('/')
    );
  }
});

// Message handling for client communication
self.addEventListener('message', (event) => {
  console.log('Service Worker: Message received', event.data);
  
  const { type, data } = event.data;
  
  switch (type) {
    case 'skip-waiting':
      self.skipWaiting();
      break;
      
    case 'cache-configuration':
      cacheConfiguration(data);
      break;
      
    case 'get-cache-info':
      getCacheInfo().then(info => {
        event.ports[0].postMessage(info);
      });
      break;
      
    case 'clear-cache':
      clearCache().then(result => {
        event.ports[0].postMessage(result);
      });
      break;
  }
});

// Cache configuration data
async function cacheConfiguration(configData) {
  try {
    const cache = await caches.open(DYNAMIC_CACHE);
    const response = new Response(JSON.stringify(configData), {
      headers: { 'Content-Type': 'application/json' }
    });
    
    await cache.put('/api/configuration/current', response);
    console.log('Service Worker: Configuration cached');
  } catch (error) {
    console.error('Service Worker: Failed to cache configuration', error);
  }
}

// Get cache information
async function getCacheInfo() {
  try {
    const cacheNames = await caches.keys();
    const cacheInfo = {};
    
    for (const cacheName of cacheNames) {
      const cache = await caches.open(cacheName);
      const keys = await cache.keys();
      cacheInfo[cacheName] = {
        size: keys.length,
        keys: keys.map(key => key.url)
      };
    }
    
    return cacheInfo;
  } catch (error) {
    console.error('Service Worker: Failed to get cache info', error);
    return { error: error.message };
  }
}

// Clear all caches
async function clearCache() {
  try {
    const cacheNames = await caches.keys();
    await Promise.all(
      cacheNames.map(cacheName => caches.delete(cacheName))
    );
    
    return { success: true, message: 'All caches cleared' };
  } catch (error) {
    console.error('Service Worker: Failed to clear cache', error);
    return { success: false, error: error.message };
  }
}

// Utility functions for IndexedDB operations
async function getPendingConfigurationSaves() {
  // Implementation would use IndexedDB to store pending saves
  return [];
}

async function removePendingConfigurationSave(id) {
  // Implementation would remove from IndexedDB
  console.log('Service Worker: Removing pending save', id);
}

// Notify all clients
async function notifyClients(data) {
  const clients = await self.clients.matchAll();
  clients.forEach(client => {
    client.postMessage(data);
  });
}

// Error handling
self.addEventListener('error', (event) => {
  console.error('Service Worker: Global error', event.error);
});

self.addEventListener('unhandledrejection', (event) => {
  console.error('Service Worker: Unhandled rejection', event.reason);
});

console.log('Service Worker: Registered successfully');
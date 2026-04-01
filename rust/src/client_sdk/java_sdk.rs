//! Java SDK generator for OpenSim client library

use std::path::PathBuf;
use anyhow::Result;

use super::{
    api_schema::{APISchema, EndpointSchema, DataTypeSchema},
    generator::{LanguageGenerator, GeneratorConfig, GeneratedFile, GeneratedFileType, GenerationResult, PackageInfo, TargetLanguage},
};

/// Java SDK generator
pub struct JavaGenerator;

impl JavaGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_pom_xml(&self, config: &GeneratorConfig) -> String {
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>

    <groupId>{}</groupId>
    <artifactId>{}</artifactId>
    <version>{}</version>
    <packaging>jar</packaging>

    <name>OpenSim Java Client</name>
    <description>OpenSim client library for Java</description>
    <url>{}</url>

    <licenses>
        <license>
            <name>{}</name>
            <distribution>repo</distribution>
        </license>
    </licenses>

    <developers>
        <developer>
            <name>{}</name>
        </developer>
    </developers>

    <properties>
        <maven.compiler.source>11</maven.compiler.source>
        <maven.compiler.target>11</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
    </properties>

    <dependencies>
        <dependency>
            <groupId>com.fasterxml.jackson.core</groupId>
            <artifactId>jackson-databind</artifactId>
            <version>2.15.2</version>
        </dependency>
        <dependency>
            <groupId>com.fasterxml.jackson.datatype</groupId>
            <artifactId>jackson-datatype-jsr310</artifactId>
            <version>2.15.2</version>
        </dependency>
        <dependency>
            <groupId>org.apache.httpcomponents</groupId>
            <artifactId>httpclient</artifactId>
            <version>4.5.14</version>
        </dependency>
        <dependency>
            <groupId>org.slf4j</groupId>
            <artifactId>slf4j-api</artifactId>
            <version>2.0.7</version>
        </dependency>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
    </dependencies>

    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-compiler-plugin</artifactId>
                <version>3.11.0</version>
                <configuration>
                    <source>11</source>
                    <target>11</target>
                </configuration>
            </plugin>
        </plugins>
    </build>
</project>
"#,
            config.namespace.as_deref().unwrap_or("org.opensim"),
            config.package_name,
            config.package_version,
            config.repository_url.as_deref().unwrap_or("https://github.com/opensim/opensim-next"),
            config.license,
            config.author
        )
    }
}

impl LanguageGenerator for JavaGenerator {
    fn generate_client(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<GenerationResult> {
        let mut generated_files = Vec::new();
        let package_name = config.namespace.as_deref().unwrap_or("org.opensim.client");

        // Generate main client class
        let client_content = format!(r#"package {};

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import org.apache.http.client.methods.CloseableHttpResponse;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.client.methods.HttpPost;
import org.apache.http.entity.StringEntity;
import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.impl.client.HttpClients;
import org.apache.http.util.EntityUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.time.LocalDateTime;
import java.util.List;

/**
 * OpenSim API client for Java
 */
public class OpenSimClient implements AutoCloseable {{
    private static final Logger logger = LoggerFactory.getLogger(OpenSimClient.class);
    
    private final CloseableHttpClient httpClient;
    private final ObjectMapper objectMapper;
    private final String baseUrl;
    private String accessToken;

    /**
     * Creates a new OpenSim client
     * @param baseUrl The base URL of the OpenSim API
     */
    public OpenSimClient(String baseUrl) {{
        this.baseUrl = baseUrl.replaceAll("/$", "");
        this.httpClient = HttpClients.createDefault();
        this.objectMapper = new ObjectMapper();
        this.objectMapper.registerModule(new JavaTimeModule());
    }}

    /**
     * Sets the access token for authentication
     * @param token The access token
     */
    public void setAccessToken(String token) {{
        this.accessToken = token;
    }}

    /**
     * Authenticates with username and password
     * @param username Username
     * @param password Password
     * @return Authentication response
     * @throws IOException If the request fails
     */
    public AuthResponse authenticate(String username, String password) throws IOException {{
        AuthRequest request = new AuthRequest(username, password);
        
        HttpPost post = new HttpPost(baseUrl + "/auth/login");
        post.setHeader("Content-Type", "application/json");
        post.setEntity(new StringEntity(objectMapper.writeValueAsString(request)));

        try (CloseableHttpResponse response = httpClient.execute(post)) {{
            String responseBody = EntityUtils.toString(response.getEntity());
            
            if (response.getStatusLine().getStatusCode() != 200) {{
                throw new IOException("Authentication failed: " + response.getStatusLine());
            }}
            
            AuthResponse authResponse = objectMapper.readValue(responseBody, AuthResponse.class);
            setAccessToken(authResponse.getAccessToken());
            return authResponse;
        }}
    }}

    /**
     * Gets user profile by ID
     * @param userId User ID
     * @return User profile
     * @throws IOException If the request fails
     */
    public UserProfile getUserProfile(String userId) throws IOException {{
        HttpGet get = new HttpGet(baseUrl + "/users/" + userId);
        addAuthHeader(get);

        try (CloseableHttpResponse response = httpClient.execute(get)) {{
            String responseBody = EntityUtils.toString(response.getEntity());
            
            if (response.getStatusLine().getStatusCode() != 200) {{
                throw new IOException("Request failed: " + response.getStatusLine());
            }}
            
            return objectMapper.readValue(responseBody, UserProfile.class);
        }}
    }}

    /**
     * Lists available regions
     * @param limit Maximum number of regions to return
     * @return List of regions
     * @throws IOException If the request fails
     */
    public List<Region> listRegions(Integer limit) throws IOException {{
        String url = baseUrl + "/regions";
        if (limit != null) {{
            url += "?limit=" + limit;
        }}
        
        HttpGet get = new HttpGet(url);
        addAuthHeader(get);

        try (CloseableHttpResponse response = httpClient.execute(get)) {{
            String responseBody = EntityUtils.toString(response.getEntity());
            
            if (response.getStatusLine().getStatusCode() != 200) {{
                throw new IOException("Request failed: " + response.getStatusLine());
            }}
            
            RegionListResponse regionResponse = objectMapper.readValue(responseBody, RegionListResponse.class);
            return regionResponse.getRegions();
        }}
    }}

    private void addAuthHeader(org.apache.http.HttpMessage request) {{
        if (accessToken != null) {{
            request.setHeader("Authorization", "Bearer " + accessToken);
        }}
    }}

    @Override
    public void close() throws IOException {{
        if (httpClient != null) {{
            httpClient.close();
        }}
    }}

    // Data classes
    public static class AuthRequest {{
        @JsonProperty("username")
        private String username;
        
        @JsonProperty("password")
        private String password;

        public AuthRequest() {{}}

        public AuthRequest(String username, String password) {{
            this.username = username;
            this.password = password;
        }}

        // Getters and setters
        public String getUsername() {{ return username; }}
        public void setUsername(String username) {{ this.username = username; }}
        public String getPassword() {{ return password; }}
        public void setPassword(String password) {{ this.password = password; }}
    }}

    public static class AuthResponse {{
        @JsonProperty("access_token")
        private String accessToken;
        
        @JsonProperty("refresh_token")
        private String refreshToken;
        
        @JsonProperty("expires_in")
        private int expiresIn;
        
        @JsonProperty("token_type")
        private String tokenType;

        // Getters and setters
        public String getAccessToken() {{ return accessToken; }}
        public void setAccessToken(String accessToken) {{ this.accessToken = accessToken; }}
        public String getRefreshToken() {{ return refreshToken; }}
        public void setRefreshToken(String refreshToken) {{ this.refreshToken = refreshToken; }}
        public int getExpiresIn() {{ return expiresIn; }}
        public void setExpiresIn(int expiresIn) {{ this.expiresIn = expiresIn; }}
        public String getTokenType() {{ return tokenType; }}
        public void setTokenType(String tokenType) {{ this.tokenType = tokenType; }}
    }}

    public static class UserProfile {{
        @JsonProperty("id")
        private String id;
        
        @JsonProperty("username")
        private String username;
        
        @JsonProperty("email")
        private String email;
        
        @JsonProperty("created_at")
        private LocalDateTime createdAt;
        
        @JsonProperty("last_login")
        private LocalDateTime lastLogin;

        // Getters and setters
        public String getId() {{ return id; }}
        public void setId(String id) {{ this.id = id; }}
        public String getUsername() {{ return username; }}
        public void setUsername(String username) {{ this.username = username; }}
        public String getEmail() {{ return email; }}
        public void setEmail(String email) {{ this.email = email; }}
        public LocalDateTime getCreatedAt() {{ return createdAt; }}
        public void setCreatedAt(LocalDateTime createdAt) {{ this.createdAt = createdAt; }}
        public LocalDateTime getLastLogin() {{ return lastLogin; }}
        public void setLastLogin(LocalDateTime lastLogin) {{ this.lastLogin = lastLogin; }}
    }}

    public static class Region {{
        @JsonProperty("id")
        private String id;
        
        @JsonProperty("name")
        private String name;
        
        @JsonProperty("position")
        private Vector2 position;
        
        @JsonProperty("size")
        private Vector2 size;
        
        @JsonProperty("online_users")
        private int onlineUsers;

        // Getters and setters
        public String getId() {{ return id; }}
        public void setId(String id) {{ this.id = id; }}
        public String getName() {{ return name; }}
        public void setName(String name) {{ this.name = name; }}
        public Vector2 getPosition() {{ return position; }}
        public void setPosition(Vector2 position) {{ this.position = position; }}
        public Vector2 getSize() {{ return size; }}
        public void setSize(Vector2 size) {{ this.size = size; }}
        public int getOnlineUsers() {{ return onlineUsers; }}
        public void setOnlineUsers(int onlineUsers) {{ this.onlineUsers = onlineUsers; }}
    }}

    public static class Vector2 {{
        @JsonProperty("x")
        private float x;
        
        @JsonProperty("y")
        private float y;

        public Vector2() {{}}

        public Vector2(float x, float y) {{
            this.x = x;
            this.y = y;
        }}

        // Getters and setters
        public float getX() {{ return x; }}
        public void setX(float x) {{ this.x = x; }}
        public float getY() {{ return y; }}
        public void setY(float y) {{ this.y = y; }}
    }}

    public static class RegionListResponse {{
        @JsonProperty("regions")
        private List<Region> regions;
        
        @JsonProperty("total")
        private int total;

        // Getters and setters
        public List<Region> getRegions() {{ return regions; }}
        public void setRegions(List<Region> regions) {{ this.regions = regions; }}
        public int getTotal() {{ return total; }}
        public void setTotal(int total) {{ this.total = total; }}
    }}
}}
"#, package_name);

        let java_file_path = format!("src/main/java/{}/OpenSimClient.java", 
                                    package_name.replace(".", "/"));
        
        generated_files.push(GeneratedFile {
            path: PathBuf::from(java_file_path),
            content: client_content,
            file_type: GeneratedFileType::SourceCode,
        });

        // Generate pom.xml
        generated_files.push(GeneratedFile {
            path: PathBuf::from("pom.xml"),
            content: self.generate_pom_xml(config),
            file_type: GeneratedFileType::Configuration,
        });

        // Generate README if requested
        if config.include_documentation {
            let readme_content = format!(r#"# {}

OpenSim client library for Java.

## Installation

Add to your `pom.xml`:

```xml
<dependency>
    <groupId>{}</groupId>
    <artifactId>{}</artifactId>
    <version>{}</version>
</dependency>
```

## Usage

```java
import {}.OpenSimClient;

try (OpenSimClient client = new OpenSimClient("https://api.opensim.org")) {{
    // Authenticate
    var auth = client.authenticate("username", "password");
    System.out.println("Authenticated: " + auth.getAccessToken());
    
    // Use the client...
}}
```

## License

{}
"#,
                config.package_name,
                config.namespace.as_deref().unwrap_or("org.opensim"),
                config.package_name,
                config.package_version,
                package_name,
                config.license
            );

            generated_files.push(GeneratedFile {
                path: PathBuf::from("README.md"),
                content: readme_content,
                file_type: GeneratedFileType::Documentation,
            });
        }

        let package_info = PackageInfo {
            name: config.package_name.clone(),
            version: config.package_version.clone(),
            description: "OpenSim client library for Java".to_string(),
            main_file: Some(PathBuf::from("src/main/java")),
            entry_points: vec!["OpenSimClient".to_string()],
            dependencies: vec![
                "jackson-databind".to_string(),
                "httpclient".to_string(),
                "slf4j-api".to_string(),
            ],
        };

        Ok(GenerationResult {
            target_language: TargetLanguage::Java,
            generated_files,
            package_info,
            warnings: vec![],
            errors: vec![],
        })
    }

    fn generate_data_types(&self, _types: &[DataTypeSchema], _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        // Java classes would be generated here
        Ok(vec![])
    }

    fn generate_api_methods(&self, _endpoints: &[EndpointSchema], _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        // API method implementations would be generated here
        Ok(vec![])
    }

    fn generate_authentication(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<GeneratedFile> {
        let content = "// Authentication methods would be generated here";
        Ok(GeneratedFile {
            path: PathBuf::from("src/main/java/org/opensim/client/Auth.java"),
            content: content.to_string(),
            file_type: GeneratedFileType::SourceCode,
        })
    }

    fn generate_configuration(&self, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        Ok(vec![GeneratedFile {
            path: PathBuf::from("pom.xml"),
            content: self.generate_pom_xml(config),
            file_type: GeneratedFileType::Configuration,
        }])
    }

    fn generate_examples(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let example_content = "// Examples would be generated here";
        Ok(vec![GeneratedFile {
            path: PathBuf::from("src/main/java/examples/BasicExample.java"),
            content: example_content.to_string(),
            file_type: GeneratedFileType::Example,
        }])
    }

    fn generate_tests(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let test_content = "// Tests would be generated here";
        Ok(vec![GeneratedFile {
            path: PathBuf::from("src/test/java/org/opensim/client/ClientTest.java"),
            content: test_content.to_string(),
            file_type: GeneratedFileType::Test,
        }])
    }

    fn get_file_extension(&self) -> &'static str {
        ".java"
    }

    fn get_package_manager_config(&self, config: &GeneratorConfig) -> Result<GeneratedFile> {
        Ok(GeneratedFile {
            path: PathBuf::from("pom.xml"),
            content: self.generate_pom_xml(config),
            file_type: GeneratedFileType::Configuration,
        })
    }
}
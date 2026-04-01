//! Ruby SDK generator for OpenSim client libraries

use super::{
    api_schema::*,
    generator::{LanguageGenerator, GeneratorConfig},
    utils::*,
};
use anyhow::Result;
use std::collections::HashMap;

/// Ruby SDK generator implementation
pub struct RubyGenerator;

impl LanguageGenerator for RubyGenerator {
    fn language_name(&self) -> &'static str {
        "Ruby"
    }

    fn file_extension(&self) -> &'static str {
        "rb"
    }

    fn generate_sdk(&self, schema: &ApiSchema, config: &GeneratorConfig) -> Result<HashMap<String, String>> {
        let mut files = HashMap::new();

        // Generate main client
        files.insert("lib/opensim/client.rb".to_string(), self.generate_client(schema, config)?);
        
        // Generate base module
        files.insert("lib/opensim.rb".to_string(), self.generate_base_module(config)?);
        
        // Generate types
        for type_def in &schema.types {
            let filename = format!("lib/opensim/models/{}.rb", to_snake_case(&type_def.name()));
            files.insert(filename, self.generate_type_definition(type_def)?);
        }
        
        // Generate authentication
        files.insert("lib/opensim/auth.rb".to_string(), self.generate_auth(schema)?);
        
        // Generate API endpoints
        for endpoint in &schema.endpoints {
            let category = endpoint.path.split('/').nth(1).unwrap_or("misc");
            let filename = format!("lib/opensim/api/{}.rb", to_snake_case(category));
            if !files.contains_key(&filename) {
                files.insert(filename, self.generate_api_category(schema, category)?);
            }
        }
        
        // Generate gemspec
        files.insert("opensim.gemspec".to_string(), self.generate_gemspec(config)?);
        
        // Generate Gemfile
        files.insert("Gemfile".to_string(), self.generate_gemfile()?);
        
        // Generate examples
        files.insert("examples/basic_usage.rb".to_string(), self.generate_examples(schema, config)?);
        
        // Generate tests
        files.insert("spec/client_spec.rb".to_string(), self.generate_tests(schema)?);
        files.insert("spec/spec_helper.rb".to_string(), self.generate_spec_helper()?);
        
        // Generate README
        files.insert("README.md".to_string(), self.generate_readme(config)?);

        Ok(files)
    }

    fn generate_type_definition(&self, type_def: &TypeDefinition) -> Result<String> {
        match type_def {
            TypeDefinition::Object(obj) => {
                let class_name = to_pascal_case(&obj.name);
                let mut result = format!(r#"# frozen_string_literal: true

module OpenSim
  module Models
    # {}
    #
    # {}
    class {}
"#, class_name, 
    obj.description.as_deref().unwrap_or(""),
    class_name);

                // Add attr_accessor for all fields
                let accessors: Vec<String> = obj.fields.iter()
                    .map(|f| format!(":{}", to_snake_case(&f.name)))
                    .collect();
                
                result.push_str(&format!("      attr_accessor {}\n\n", accessors.join(", ")));

                // Add initialize method
                result.push_str("      # Initialize new instance\n");
                result.push_str("      # @param attributes [Hash] Initial attributes\n");
                result.push_str("      def initialize(attributes = {})\n");
                
                for field in &obj.fields {
                    let field_name = to_snake_case(&field.name);
                    result.push_str(&format!("        @{} = attributes[:{}] || attributes['{}']\n", 
                        field_name, field_name, field.name));
                }
                
                result.push_str("      end\n\n");

                // Add to_hash method
                result.push_str("      # Convert to hash\n");
                result.push_str("      # @return [Hash]\n");
                result.push_str("      def to_hash\n");
                result.push_str("        {\n");
                
                for field in &obj.fields {
                    let field_name = to_snake_case(&field.name);
                    result.push_str(&format!("          '{}' => @{},\n", field.name, field_name));
                }
                
                result.push_str("        }.compact\n");
                result.push_str("      end\n\n");

                // Add from_hash class method
                result.push_str("      # Create from hash\n");
                result.push_str("      # @param hash [Hash]\n");
                result.push_str(&format!("      # @return [{}]\n", class_name));
                result.push_str("      def self.from_hash(hash)\n");
                result.push_str("        new(hash)\n");
                result.push_str("      end\n");

                result.push_str("    end\n  end\nend\n");
                Ok(result)
            }
            TypeDefinition::Enum(enum_def) => {
                let module_name = to_pascal_case(&enum_def.name);
                let mut result = format!(r#"# frozen_string_literal: true

module OpenSim
  module Models
    # {}
    #
    # {}
    module {}
"#, module_name,
    enum_def.description.as_deref().unwrap_or(""),
    module_name);

                for variant in &enum_def.variants {
                    result.push_str(&format!("      {} = '{}'\n", 
                        to_screaming_snake_case(&variant.name), 
                        variant.value));
                }

                result.push_str("\n      # Get all valid values\n");
                result.push_str("      # @return [Array<String>]\n");
                result.push_str("      def self.all\n");
                result.push_str("        [\n");
                
                for variant in &enum_def.variants {
                    result.push_str(&format!("          {},\n", to_screaming_snake_case(&variant.name)));
                }
                
                result.push_str("        ]\n");
                result.push_str("      end\n\n");

                result.push_str("      # Check if value is valid\n");
                result.push_str("      # @param value [String]\n");
                result.push_str("      # @return [Boolean]\n");
                result.push_str("      def self.valid?(value)\n");
                result.push_str("        all.include?(value)\n");
                result.push_str("      end\n");

                result.push_str("    end\n  end\nend\n");
                Ok(result)
            }
        }
    }
}

impl RubyGenerator {
    fn generate_base_module(&self, config: &GeneratorConfig) -> Result<String> {
        Ok(format!(r#"# frozen_string_literal: true

require 'net/http'
require 'json'
require 'uri'

require_relative 'opensim/client'
require_relative 'opensim/auth'
require_relative 'opensim/version'

# OpenSim Ruby SDK
#
# Generated by OpenSim SDK Generator
# Version: {}
module OpenSim
  class Error < StandardError; end
  class APIError < Error
    attr_reader :code, :details

    def initialize(message, code = nil, details = nil)
      super(message)
      @code = code
      @details = details
    end
  end
  
  class AuthenticationError < Error; end
  class ValidationError < Error; end
end
"#, config.version))
    }

    fn generate_client(&self, _schema: &ApiSchema, config: &GeneratorConfig) -> Result<String> {
        Ok(format!(r#"# frozen_string_literal: true

require 'net/http'
require 'json'
require 'uri'

module OpenSim
  # OpenSim API Client
  #
  # Generated by OpenSim SDK Generator
  # Version: {}
  class Client
    attr_reader :base_url, :api_key, :timeout
    attr_accessor :auth_manager

    # Initialize new client
    # @param base_url [String] API base URL
    # @param api_key [String] API key for authentication
    # @param timeout [Integer] Request timeout in seconds
    def initialize(base_url:, api_key: nil, timeout: 30)
      @base_url = base_url.chomp('/')
      @api_key = api_key
      @timeout = timeout
      @auth_manager = Auth.new(self)
    end

    # Make HTTP request to API
    # @param method [Symbol] HTTP method (:get, :post, :put, :delete)
    # @param path [String] API path
    # @param data [Hash] Request data
    # @param headers [Hash] Additional headers
    # @return [Hash] Parsed response
    # @raise [APIError] If request fails
    def make_request(method, path, data: nil, headers: {{}})
      uri = URI.parse("#{base_url}#{path}")
      
      http = Net::HTTP.new(uri.host, uri.port)
      http.use_ssl = uri.scheme == 'https'
      http.read_timeout = timeout
      
      request = build_request(method, uri, data, headers)
      
      response = http.request(request)
      handle_response(response)
    rescue Net::TimeoutError => e
      raise APIError.new("Request timeout: #{e.message}")
    rescue StandardError => e
      raise APIError.new("Request failed: #{e.message}")
    end

    # Test API connection
    # @return [Boolean]
    def ping
      make_request(:get, '/api/health')
      true
    rescue APIError
      false
    end

    # Get authentication manager
    # @return [Auth]
    def auth
      @auth_manager
    end

    private

    # Build HTTP request object
    # @param method [Symbol] HTTP method
    # @param uri [URI] Request URI
    # @param data [Hash] Request data
    # @param headers [Hash] Additional headers
    # @return [Net::HTTPRequest]
    def build_request(method, uri, data, headers)
      case method
      when :get
        request = Net::HTTP::Get.new(uri)
        if data && !data.empty?
          uri.query = URI.encode_www_form(data)
          request = Net::HTTP::Get.new(uri)
        end
      when :post
        request = Net::HTTP::Post.new(uri)
        set_request_body(request, data)
      when :put
        request = Net::HTTP::Put.new(uri)
        set_request_body(request, data)
      when :delete
        request = Net::HTTP::Delete.new(uri)
      else
        raise ArgumentError, "Unsupported HTTP method: #{method}"
      end

      # Set default headers
      request['Content-Type'] = 'application/json'
      request['Accept'] = 'application/json'
      request['User-Agent'] = 'opensim-ruby-sdk/{}'
      request['X-API-Key'] = api_key if api_key

      # Set additional headers
      headers.each {{|key, value| request[key] = value }}

      request
    end

    # Set request body for POST/PUT requests
    # @param request [Net::HTTPRequest] Request object
    # @param data [Hash] Request data
    def set_request_body(request, data)
      return unless data

      request.body = JSON.generate(data)
    end

    # Handle HTTP response
    # @param response [Net::HTTPResponse] HTTP response
    # @return [Hash] Parsed response data
    # @raise [APIError] If response indicates error
    def handle_response(response)
      case response.code.to_i
      when 200..299
        return {{}} if response.body.nil? || response.body.strip.empty?
        
        JSON.parse(response.body)
      when 400..499
        error_data = parse_error_response(response)
        raise APIError.new(
          error_data['message'] || 'Client error',
          response.code.to_i,
          error_data['details']
        )
      when 500..599
        error_data = parse_error_response(response)
        raise APIError.new(
          error_data['message'] || 'Server error',
          response.code.to_i,
          error_data['details']
        )
      else
        raise APIError.new("Unexpected response code: #{response.code}", response.code.to_i)
      end
    end

    # Parse error response
    # @param response [Net::HTTPResponse] HTTP response
    # @return [Hash] Parsed error data
    def parse_error_response(response)
      return {{}} if response.body.nil? || response.body.strip.empty?
      
      JSON.parse(response.body)
    rescue JSON::ParserError
      {{ 'message' => response.body }}
    end
  end
end
"#, config.version, config.version))
    }

    fn generate_auth(&self, _schema: &ApiSchema) -> Result<String> {
        Ok(r#"# frozen_string_literal: true

module OpenSim
  # Authentication manager
  class Auth
    attr_reader :client
    attr_accessor :token, :expires_at

    # Initialize auth manager
    # @param client [Client] API client instance
    def initialize(client)
      @client = client
      @token = nil
      @expires_at = nil
    end

    # Login with username and password
    # @param username [String] Username
    # @param password [String] Password
    # @return [Hash] Login response
    # @raise [AuthenticationError] If login fails
    def login(username, password)
      response = client.make_request(:post, '/api/auth/login', data: {
        username: username,
        password: password
      })

      @token = response.dig('data', 'token')
      @expires_at = response.dig('data', 'expires_at')

      response['data']
    rescue APIError => e
      raise AuthenticationError, "Login failed: #{e.message}"
    end

    # Logout
    # @return [void]
    # @raise [AuthenticationError] If logout fails
    def logout
      raise AuthenticationError, 'Not authenticated' unless authenticated?

      client.make_request(:post, '/api/auth/logout')
      @token = nil
      @expires_at = nil
    rescue APIError => e
      raise AuthenticationError, "Logout failed: #{e.message}"
    end

    # Check if authenticated
    # @return [Boolean]
    def authenticated?
      !token.nil? && (expires_at.nil? || expires_at > Time.now.to_i)
    end

    # Set token manually
    # @param token [String] Authentication token
    # @param expires_at [Integer] Token expiration timestamp
    def set_token(token, expires_at = nil)
      @token = token
      @expires_at = expires_at
    end

    # Get current token
    # @return [String, nil]
    def current_token
      authenticated? ? token : nil
    end
  end
end
"#.to_string())
    }

    fn generate_api_category(&self, schema: &ApiSchema, category: &str) -> Result<String> {
        let class_name = to_pascal_case(category);
        let mut result = format!(r#"# frozen_string_literal: true

module OpenSim
  module API
    # {} API endpoints
    class {}
      attr_reader :client

      # Initialize API handler
      # @param client [Client] API client instance
      def initialize(client)
        @client = client
      end

"#, class_name, class_name);

        let endpoints: Vec<_> = schema.endpoints.iter()
            .filter(|e| e.path.starts_with(&format!("/{}", category)))
            .collect();

        for endpoint in endpoints {
            result.push_str(&self.generate_endpoint_method(endpoint)?);
        }

        result.push_str("    end\n  end\nend\n");
        Ok(result)
    }

    fn generate_endpoint_method(&self, endpoint: &ApiEndpoint) -> Result<String> {
        let method_name = self.generate_method_name(&endpoint.path, &endpoint.method);
        
        let mut params = Vec::new();
        let mut path_params = Vec::new();
        
        // Add path parameters
        for param in &endpoint.parameters {
            if param.location == ParameterLocation::Path {
                params.push(to_snake_case(&param.name));
                path_params.push(param.name.clone());
            }
        }
        
        // Add query parameters
        let has_query_params = endpoint.parameters.iter().any(|p| p.location == ParameterLocation::Query);
        if has_query_params {
            params.push("query_params = {}".to_string());
        }
        
        // Add request body
        if endpoint.request_body.is_some() {
            params.push("data = {}".to_string());
        }
        
        let params_str = params.join(", ");

        Ok(format!(r#"      # {} {}
      # 
{}
      # @return [Hash] API response
      # @raise [APIError] If request fails
      def {}({})
        path = '{}'
{}
        
        {}
      end

"#, 
            endpoint.method.to_uppercase(),
            endpoint.summary.as_deref().unwrap_or(&endpoint.path),
            endpoint.parameters.iter()
                .map(|p| format!("      # @param {} [{}] {}", 
                    to_snake_case(&p.name), 
                    self.map_type_to_ruby(&p.param_type),
                    p.description.as_deref().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n"),
            method_name,
            params_str,
            endpoint.path,
            self.generate_path_replacements(&path_params),
            self.generate_request_call(endpoint, has_query_params)
        ))
    }

    fn generate_method_name(&self, path: &str, method: &str) -> String {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty() && !s.starts_with('{')).collect();
        let name = parts.join("_");
        to_snake_case(&format!("{}_{}", method, name))
    }

    fn generate_path_replacements(&self, path_params: &[String]) -> String {
        if path_params.is_empty() {
            return String::new();
        }
        
        let mut replacements = String::new();
        for param in path_params {
            replacements.push_str(&format!("        path = path.gsub('{{{}}}', {})\n", 
                param, to_snake_case(param)));
        }
        
        replacements
    }

    fn generate_request_call(&self, endpoint: &ApiEndpoint, has_query_params: bool) -> String {
        let data_param = if endpoint.request_body.is_some() {
            "data: data"
        } else if has_query_params {
            "data: query_params"
        } else {
            ""
        };

        format!("client.make_request(:{}, path{}{})", 
            endpoint.method.to_lowercase(),
            if !data_param.is_empty() { ", " } else { "" },
            data_param)
    }

    fn generate_gemspec(&self, config: &GeneratorConfig) -> Result<String> {
        Ok(format!(r##"# frozen_string_literal: true

require_relative 'lib/opensim/version'

Gem::Specification.new do |spec|
  spec.name          = '{}'
  spec.version       = OpenSim::VERSION
  spec.authors       = ['OpenSim Contributors']
  spec.email         = ['developers@opensim.org']

  spec.summary       = 'Official Ruby SDK for the OpenSim API'
  spec.description   = 'A comprehensive Ruby client library for interacting with the OpenSim virtual world platform API.'
  spec.homepage      = 'https://github.com/{}/{}'
  spec.license       = '{}'

  spec.required_ruby_version = '>= 2.7.0'

  spec.metadata['homepage_uri'] = spec.homepage
  spec.metadata['source_code_uri'] = spec.homepage
  spec.metadata['changelog_uri'] = "#{spec.homepage}/blob/main/CHANGELOG.md"

  # Specify which files should be added to the gem when it is released.
  spec.files = Dir['lib/**/*', 'README.md', 'LICENSE', 'CHANGELOG.md']
  spec.bindir        = 'exe'
  spec.executables   = spec.files.grep(%r{{^exe/}}) {{ |f| File.basename(f) }}
  spec.require_paths = ['lib']

  # Runtime dependencies
  spec.add_dependency 'json', '~> 2.0'

  # Development dependencies
  spec.add_development_dependency 'bundler', '~> 2.0'
  spec.add_development_dependency 'rake', '~> 13.0'
  spec.add_development_dependency 'rspec', '~> 3.0'
  spec.add_development_dependency 'webmock', '~> 3.0'
  spec.add_development_dependency 'vcr', '~> 6.0'
  spec.add_development_dependency 'rubocop', '~> 1.0'
  spec.add_development_dependency 'yard', '~> 0.9'
end
"##, 
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.organization.as_deref().unwrap_or("opensim"),
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.license.as_deref().unwrap_or("MIT")))
    }

    fn generate_gemfile(&self) -> Result<String> {
        Ok(r#"# frozen_string_literal: true

source 'https://rubygems.org'

# Specify your gem's dependencies in opensim.gemspec
gemspec

gem 'rake', '~> 13.0'
gem 'rspec', '~> 3.0'
gem 'webmock', '~> 3.0'
gem 'vcr', '~> 6.0'
gem 'rubocop', '~> 1.0'
gem 'yard', '~> 0.9'
"#.to_string())
    }

    fn generate_examples(&self, _schema: &ApiSchema, _config: &GeneratorConfig) -> Result<String> {
        Ok(r#"#!/usr/bin/env ruby
# frozen_string_literal: true

require_relative '../lib/opensim'

# Create client
client = OpenSim::Client.new(
  base_url: 'https://api.opensim.org',
  api_key: 'your-api-key',
  timeout: 30
)

begin
  # Test connection
  if client.ping
    puts 'Successfully connected to OpenSim API'
  end

  # Login
  login_result = client.auth.login('username', 'password')
  puts "Logged in successfully. Token expires at: #{login_result['expires_at']}"

  # Example API calls would go here...

  # Logout
  client.auth.logout
  puts 'Logged out successfully'

rescue OpenSim::APIError => e
  puts "API Error: #{e.message} (Code: #{e.code})"
rescue OpenSim::AuthenticationError => e
  puts "Authentication Error: #{e.message}"
rescue OpenSim::Error => e
  puts "Error: #{e.message}"
end
"#.to_string())
    }

    fn generate_tests(&self, _schema: &ApiSchema) -> Result<String> {
        Ok(r#"# frozen_string_literal: true

require 'spec_helper'

RSpec.describe OpenSim::Client do
  let(:client) do
    OpenSim::Client.new(
      base_url: 'https://api.example.com',
      api_key: 'test-key'
    )
  end

  describe '#initialize' do
    it 'creates a new client with correct attributes' do
      expect(client.base_url).to eq('https://api.example.com')
      expect(client.api_key).to eq('test-key')
      expect(client.timeout).to eq(30)
    end

    it 'strips trailing slash from base_url' do
      client_with_slash = OpenSim::Client.new(base_url: 'https://api.example.com/')
      expect(client_with_slash.base_url).to eq('https://api.example.com')
    end
  end

  describe '#ping' do
    it 'returns true when API is reachable' do
      stub_request(:get, 'https://api.example.com/api/health')
        .to_return(status: 200, body: '{}')

      expect(client.ping).to be true
    end

    it 'returns false when API is not reachable' do
      stub_request(:get, 'https://api.example.com/api/health')
        .to_return(status: 500)

      expect(client.ping).to be false
    end
  end

  describe '#auth' do
    it 'returns auth manager instance' do
      expect(client.auth).to be_an_instance_of(OpenSim::Auth)
    end
  end
end
"#.to_string())
    }

    fn generate_spec_helper(&self) -> Result<String> {
        Ok(r#"# frozen_string_literal: true

require 'bundler/setup'
require 'opensim'
require 'webmock/rspec'
require 'vcr'

# Configure WebMock
WebMock.disable_net_connect!(allow_localhost: true)

# Configure VCR
VCR.configure do |config|
  config.cassette_library_dir = 'spec/vcr_cassettes'
  config.hook_into :webmock
  config.configure_rspec_metadata!
  config.default_cassette_options = {
    record: :once,
    re_record_interval: 7.days
  }
end

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = '.rspec_status'

  # Disable RSpec exposing methods globally on Module and main
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end

  # Use VCR for HTTP interactions
  config.around(:each, :vcr) do |example|
    name = example.metadata[:full_description].split(/\s+/, 2).join('/').underscore.gsub(/[^\w\/]+/, '_')
    VCR.use_cassette(name) { example.call }
  end
end
"#.to_string())
    }

    fn generate_readme(&self, config: &GeneratorConfig) -> Result<String> {
        Ok(format!(r#"# OpenSim Ruby SDK

[![Gem Version](https://badge.fury.io/rb/{}.svg)](https://badge.fury.io/rb/{})
[![Ruby](https://github.com/{}/{}/actions/workflows/ruby.yml/badge.svg)](https://github.com/{}/{}/actions/workflows/ruby.yml)

Official Ruby client library for the OpenSim API.

## Installation

Add this line to your application's Gemfile:

```ruby
gem '{}'
```

And then execute:

```bash
bundle install
```

Or install it yourself as:

```bash
gem install {}
```

## Quick Start

```ruby
require 'opensim'

# Create client
client = OpenSim::Client.new(
  base_url: 'https://api.opensim.org',
  api_key: 'your-api-key'
)

# Test connection
if client.ping
  puts 'Connected to OpenSim API!'
end

# Login
login_result = client.auth.login('username', 'password')
puts "Logged in successfully!"
```

## Features

- **Clean API**: Intuitive Ruby interfaces following Ruby conventions
- **Type Safety**: Full documentation with YARD
- **Error Handling**: Comprehensive exception hierarchy
- **Authentication**: Built-in support for API key and token authentication
- **Testing**: RSpec test suite with VCR for HTTP interactions
- **Standards Compliance**: Follows Ruby community standards and best practices

## Documentation

Full API documentation is available at [https://docs.opensim.org](https://docs.opensim.org).

Generate local documentation:

```bash
yard doc
```

## Testing

```bash
bundle exec rspec
```

Run with coverage:

```bash
bundle exec rspec --format documentation
```

## Development

After checking out the repo, run `bin/setup` to install dependencies. Then, run `rake spec` to run the tests.

To install this gem onto your local machine, run `bundle exec rake install`.

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/{}/{}.

## License

The gem is available as open source under the terms of the [{}](https://opensource.org/licenses/{}).
"#, 
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.organization.as_deref().unwrap_or("opensim"),
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.organization.as_deref().unwrap_or("opensim"),
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.organization.as_deref().unwrap_or("opensim"),
            config.package_name.as_deref().unwrap_or("opensim-ruby-sdk"),
            config.license.as_deref().unwrap_or("MIT"),
            config.license.as_deref().unwrap_or("MIT")))
    }

    fn map_type_to_ruby(&self, data_type: &DataType) -> String {
        match data_type {
            DataType::String => "String".to_string(),
            DataType::Integer => "Integer".to_string(),
            DataType::Number => "Float".to_string(),
            DataType::Boolean => "Boolean".to_string(),
            DataType::Array(_) => "Array".to_string(),
            DataType::Object(_) => "Hash".to_string(),
            DataType::Optional(inner) => format!("{}, nil", self.map_type_to_ruby(inner)),
            DataType::Map(_) => "Hash".to_string(),
        }
    }
}

fn to_screaming_snake_case(input: &str) -> String {
    to_snake_case(input).to_uppercase()
}
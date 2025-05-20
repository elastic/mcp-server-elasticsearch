#!/bin/bash

# Configuration
ES_HOST="${ES_URL:-http://localhost:9200}"
ES_USERNAME="${ES_USERNAME:-elastic}"
ES_PASSWORD="${ES_PASSWORD:-}"

# Check if password is provided
if [ -z "$ES_PASSWORD" ]; then
  echo "ERROR: Elasticsearch password not provided. Set the ES_PASSWORD environment variable."
  echo "Usage: ES_PASSWORD=your_password ./create_mock_data.sh"
  exit 1
fi

ES_AUTH="$ES_USERNAME:$ES_PASSWORD"

echo "Creating mock data in Elasticsearch at $ES_HOST..."

# Create 'customers' index
echo "Creating 'customers' index..."
curl -X PUT -u "$ES_AUTH" "$ES_HOST/customers" -H 'Content-Type: application/json' -d '{
  "mappings": {
    "properties": {
      "name": { "type": "text" },
      "email": { "type": "keyword" },
      "phone": { "type": "keyword" },
      "address": { "type": "text" },
      "age": { "type": "integer" },
      "registration_date": { "type": "date" }
    }
  }
}'
echo -e "\n"

# Create 'products' index
echo "Creating 'products' index..."
curl -X PUT -u "$ES_AUTH" "$ES_HOST/products" -H 'Content-Type: application/json' -d '{
  "mappings": {
    "properties": {
      "name": { "type": "text" },
      "description": { "type": "text" },
      "category": { "type": "keyword" },
      "price": { "type": "float" },
      "in_stock": { "type": "boolean" },
      "tags": { "type": "keyword" }
    }
  }
}'
echo -e "\n"

# Create 'orders' index
echo "Creating 'orders' index..."
curl -X PUT -u "$ES_AUTH" "$ES_HOST/orders" -H 'Content-Type: application/json' -d '{
  "mappings": {
    "properties": {
      "customer_id": { "type": "keyword" },
      "order_date": { "type": "date" },
      "status": { "type": "keyword" },
      "total_amount": { "type": "float" },
      "items": { 
        "type": "nested",
        "properties": {
          "product_id": { "type": "keyword" },
          "quantity": { "type": "integer" },
          "price": { "type": "float" }
        }
      }
    }
  }
}'
echo -e "\n"

# Add customer documents
echo "Adding customer documents..."
curl -X POST -u "$ES_AUTH" "$ES_HOST/customers/_doc/1001" -H 'Content-Type: application/json' -d '{
  "name": "John Smith",
  "email": "john.smith@example.com",
  "phone": "555-123-4567",
  "address": "123 Main St, Springfield, IL",
  "age": 42,
  "registration_date": "2023-01-15"
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/customers/_doc/1002" -H 'Content-Type: application/json' -d '{
  "name": "Sarah Johnson",
  "email": "sarah.j@example.com",
  "phone": "555-987-6543",
  "address": "456 Oak Ave, Rivertown, CA",
  "age": 35,
  "registration_date": "2023-03-22"
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/customers/_doc/1003" -H 'Content-Type: application/json' -d '{
  "name": "Michael Brown",
  "email": "mbrown@example.com",
  "phone": "555-456-7890",
  "address": "789 Pine Rd, Lakeside, TX",
  "age": 29,
  "registration_date": "2023-05-08"
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/customers/_doc/1004" -H 'Content-Type: application/json' -d '{
  "name": "Emily Davis",
  "email": "emily.davis@example.com",
  "phone": "555-321-6789",
  "address": "101 Cedar Ln, Mountainview, WA",
  "age": 51,
  "registration_date": "2023-07-17"
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/customers/_doc/1005" -H 'Content-Type: application/json' -d '{
  "name": "David Wilson",
  "email": "dwilson@example.com",
  "phone": "555-789-0123",
  "address": "202 Elm St, Valleytown, NY",
  "age": 38,
  "registration_date": "2023-09-30"
}'
echo -e "\n"

# Add product documents
echo "Adding product documents..."
curl -X POST -u "$ES_AUTH" "$ES_HOST/products/_doc/2001" -H 'Content-Type: application/json' -d '{
  "name": "Smart TV 55-inch",
  "description": "55-inch 4K Ultra HD Smart TV with HDR and voice assistant compatibility",
  "category": "Electronics",
  "price": 699.99,
  "in_stock": true,
  "tags": ["television", "smart home", "entertainment"]
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/products/_doc/2002" -H 'Content-Type: application/json' -d '{
  "name": "Wireless Noise-Cancelling Headphones",
  "description": "Premium wireless headphones with active noise cancellation and 30 hours of battery life",
  "category": "Audio",
  "price": 249.99,
  "in_stock": true,
  "tags": ["audio", "wireless", "headphones"]
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/products/_doc/2003" -H 'Content-Type: application/json' -d '{
  "name": "Ergonomic Office Chair",
  "description": "Fully adjustable ergonomic office chair with lumbar support and breathable mesh back",
  "category": "Furniture",
  "price": 199.99,
  "in_stock": false,
  "tags": ["office", "chair", "furniture"]
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/products/_doc/2004" -H 'Content-Type: application/json' -d '{
  "name": "Stainless Steel Cookware Set",
  "description": "10-piece stainless steel cookware set with non-stick coating and heat-resistant handles",
  "category": "Kitchen",
  "price": 149.99,
  "in_stock": true,
  "tags": ["kitchen", "cookware", "cooking"]
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/products/_doc/2005" -H 'Content-Type: application/json' -d '{
  "name": "Fitness Smartwatch",
  "description": "Waterproof smartwatch with heart rate monitor, GPS, and 7-day battery life",
  "category": "Wearables",
  "price": 179.99,
  "in_stock": true,
  "tags": ["fitness", "smartwatch", "wearable"]
}'
echo -e "\n"

# Add order documents
echo "Adding order documents..."
curl -X POST -u "$ES_AUTH" "$ES_HOST/orders/_doc/3001" -H 'Content-Type: application/json' -d '{
  "customer_id": "1001",
  "order_date": "2024-01-10",
  "status": "completed",
  "total_amount": 949.98,
  "items": [
    {
      "product_id": "2001",
      "quantity": 1,
      "price": 699.99
    },
    {
      "product_id": "2004",
      "quantity": 1,
      "price": 149.99
    }
  ]
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/orders/_doc/3002" -H 'Content-Type: application/json' -d '{
  "customer_id": "1002",
  "order_date": "2024-02-15",
  "status": "completed",
  "total_amount": 249.99,
  "items": [
    {
      "product_id": "2002",
      "quantity": 1,
      "price": 249.99
    }
  ]
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/orders/_doc/3003" -H 'Content-Type: application/json' -d '{
  "customer_id": "1003",
  "order_date": "2024-03-22",
  "status": "processing",
  "total_amount": 359.98,
  "items": [
    {
      "product_id": "2005",
      "quantity": 2,
      "price": 179.99
    }
  ]
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/orders/_doc/3004" -H 'Content-Type: application/json' -d '{
  "customer_id": "1004",
  "order_date": "2024-04-05",
  "status": "shipped",
  "total_amount": 199.99,
  "items": [
    {
      "product_id": "2003",
      "quantity": 1,
      "price": 199.99
    }
  ]
}'
echo -e "\n"

curl -X POST -u "$ES_AUTH" "$ES_HOST/orders/_doc/3005" -H 'Content-Type: application/json' -d '{
  "customer_id": "1005",
  "order_date": "2024-05-18",
  "status": "processing",
  "total_amount": 1099.97,
  "items": [
    {
      "product_id": "2001",
      "quantity": 1,
      "price": 699.99
    },
    {
      "product_id": "2002",
      "quantity": 1,
      "price": 249.99
    },
    {
      "product_id": "2004",
      "quantity": 1,
      "price": 149.99
    }
  ]
}'
echo -e "\n"

echo "Refreshing indices..."
curl -X POST -u "$ES_AUTH" "$ES_HOST/_refresh"
echo -e "\n"

echo "Mock data creation completed!"
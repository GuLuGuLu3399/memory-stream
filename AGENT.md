# AI Agent Instructions (AGENT.md)

## 1. Project Overview
This repository contains a modern Knowledge Management System and Web Reader (Bifrost architecture). The system is designed for high performance, real-time synchronization, and offline capabilities. It handles complex knowledge graph relationships, markdown parsing, and large-scale data rendering.

## 2. System Architecture & Tech Stack
The project adopts a polyglot microservices/modular architecture:

- **Frontend (Web Reader/UI):** Vue.js (Vue 3). Focuses on complex UI/UX (e.g., Virtual Lists, Entrance Animations), real-time data visualization (StatsWidgets, sparklines), and offline support (IndexedDB).
- **Backend Orchestration:** Go. Handles RESTful API endpoints, business logic, WebSocket connections for real-time graph synchronization, and slow SQL query logging (threshold: 200ms).
- **Core Engine (High-Performance Compute):** Rust. Packaged as a workspace, responsible for CPU-intensive tasks such as Markdown parsing, image compression, and local draft management.
- **Database & Middleware:** PostgreSQL/MySQL for persistence, NATS for messaging, and gRPC for internal service communication.

## 3. General AI Coding Guidelines
When generating, refactoring, or reviewing code in this repository, the AI Agent MUST adhere to the following principles:

### 3.1. Code Quality & Design
- **Single Responsibility Principle:** Keep functions small and focused.
- **Error Handling:** Never swallow errors. In Go, return errors explicitly. In Rust, use `Result` and `Option` properly. In Vue/TS, handle Promise rejections and API failures gracefully.
- **Performance First:** Be mindful of memory allocation and blocking operations. Use asynchronous patterns where appropriate.
- **Comments & Documentation:** Document complex logic, API endpoints, and WebSocket event payloads. Use standard docstring formats (e.g., Godoc, Rustdoc, JSDoc).

### 3.2. Go (Backend) Guidelines
- Use standard Go idiomatic patterns.
- Ensure all database queries are optimized. Pay special attention to the slow query logger; avoid N+1 query problems.
- WebSocket implementations must handle disconnects, heartbeats (ping/pong), and concurrent map writes safely (use `sync.RWMutex` or channels).
- Strictly define JSON/WebSocket payloads using Go structs with proper tags.

### 3.3. Rust (Core Engine) Guidelines
- Utilize Cargo workspaces correctly. Keep modules decoupled (e.g., separation between `markdown-parser` and `image-compressor`).
- Maximize zero-cost abstractions and avoid unnecessary `.clone()` calls. 
- Ensure memory safety and handle edge cases in Markdown AST parsing.
- Provide safe FFI bindings or well-defined CLI/gRPC interfaces if communicating directly with the Go layer.

### 3.4. Vue.js & TypeScript (Frontend) Guidelines
- Use `<script setup>` syntax and Composition API (`useGraphSync.ts`, `useOfflineCache.ts`).
- **Offline-First:** When fetching data, always consider the IndexedDB cache fallback.
- **Reactivity:** Be careful with large reactive objects (like complex graph nodes). Use `shallowRef` where deep reactivity is unnecessary for performance.
- **Components:** Keep UI components (like `EntranceAnimation.vue`, `StatsWidget.vue`) decoupled from business logic. Pass data via props and emit events.

## 4. Specific Business Logic Rules
- **Knowledge Graph Sync:** Any updates to `cards` or `edges` must be broadcasted via WebSocket to all connected clients. Ensure idempotency in event handling.
- **Offline Caching:** Card summaries should be aggressively cached in IndexedDB upon fetching to ensure immediate access during network interruptions.
- **Markdown Handling:** The Go server should delegate heavy markdown parsing and AST transformations to the Rust engine rather than processing it directly.

## 5. Prohibited Patterns
- DO NOT generate code with inline CSS; use scoped styles or the defined CSS framework.
- DO NOT use `any` in TypeScript; explicitly define interfaces for all data structures.
- DO NOT introduce synchronous file I/O operations in the Go WebSocket request lifecycle.
- DO NOT bypass the API layer to interact with the database directly from the frontend.

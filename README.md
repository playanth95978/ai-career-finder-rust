# 🚀 Job Search AI (`job-search-rust`)

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7-blue.svg?style=flat-square)](https://github.com/tokio-rs/axum)
[![Diesel](https://img.shields.io/badge/Diesel-PostgreSQL-red.svg?style=flat-square&logo=postgresql)](https://diesel.rs/)
[![Angular](https://img.shields.io/badge/Angular-Frontend-DD0031.svg?style=flat-square&logo=angular)](https://angular.dev/)
[![Docker](https://img.shields.io/badge/Docker-Enabled-2496ED.svg?style=flat-square&logo=docker)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-Proprietary-gray.svg?style=flat-square)](#)

> **Job Search AI** is an intelligent platform for job offer aggregation, predictive AI matching, tailored resume generation, and automated job applications.
> Engineered for maximum performance and high reliability with an asynchronous **Rust (Axum + Diesel)** backend and a responsive **Angular** frontend.

---

## 📑 Table of Contents

- [✨ Key Features](#-key-features)
- [🏛️ Architecture & Data Model](#️-architecture--data-model)
- [🛠️ Tech Stack](#️-tech-stack)
- [📋 Prerequisites](#-prerequisites)
- [🚀 Quick Start](#-quick-start)
- [🌐 Endpoints & API Documentation](#-endpoints--api-documentation)
- [⚙️ Configuration & Environment Variables](#️-configuration--environment-variables)
- [🧪 Testing & Quality](#-testing--quality)
- [📦 Deployment & Docker](#-deployment--docker)
- [📂 Project Structure](#-project-structure)

---

## ✨ Key Features

- 🔍 **Multi-Source Aggregation & Indexing**: Unified collection of job postings across multiple platforms (Greenhouse, Lever, SmartRecruiters, Ashby, RemoteOK, France Travail, LinkedIn, etc.) with content hash deduplication (`content_hash`).
- 🧠 **Vectorization & AI Embeddings**: Semantic embedding computation on job postings and candidate profiles for high-precision vector similarity matching.
- 🎯 **Opportunity Radar & Auto-Scoring**: Proactive detection of matching offers, relevance score calculation (`match_score`), and automatic generation of personalized pitch arguments (*"Why You"*).
- 📄 **Resume Manager & Versioning**: Structured editing, template management, dynamic versioning (`cv_resume_version`), and custom tailoring for targeted job offers (`offer_tailored_resume`).
- 🤖 **Intelligent Auto-Apply**: Automated or semi-supervised application pipeline (validation mode or full-auto mode) with strict daily quotas and exclusion filters.
- 💬 **Conversational Assistant & Dedicated AI**: Interactive multi-context chat module (Code, Jira, Confluence, PDF) to analyze offers and prepare for interviews.
- 🔐 **Security & RBAC**: JWT authentication, robust password hashing with Argon2, and role-based access control (Admin/User).

---

## 🏛️ Architecture & Data Model

The database schema (`schema.rs`) structures the functional core of the platform:

```
                        ┌──────────────────┐
                        │      users       │
                        └────────┬─────────┘
                                 │ 1..N
    ┌────────────────────────────┼────────────────────────────┐
    │                            │                            │
    ▼                            ▼                            ▼
┌──────────────────┐   ┌──────────────────┐         ┌──────────────────┐
│candidate_profile │   │ user_preference  │         │auto_apply_config │
└────────┬─────────┘   └──────────────────┘         └──────────────────┘
         │
         │ 1..N
         ▼
┌──────────────────┐   match_score   ┌──────────────────┐
│ job_application  │ ◄─────────────► │    job_offer     │
└──────────────────┘                 └────────┬─────────┘
                                              │
         ┌────────────────────────────────────┼────────────────────────────────────┐
         │                                    │                                    │
         ▼                                    ▼                                    ▼
┌──────────────────┐                ┌──────────────────┐                 ┌───────────────────────┐
│    radar_hit     │                │offer_positioning │                 │ offer_tailored_resume │
│ (why_you, score) │                │   (AI analysis)  │                 │ (tailored CV per job) │
└──────────────────┘                └──────────────────┘                 └───────────────────────┘
```

### Main Entities (`schema.rs`)

| Table / Entity | Description & Domain Role |
| :--- | :--- |
| `job_offer` | Centralized job offers with contract metadata, salary ranges, remote status, embedding status, and reindexing tracking. |
| `candidate_profile` | Enriched candidate profile: skills, experience history, education, certifications, raw markdown CV, and semantic vector. |
| `job_application` | Application lifecycle tracking (`DRAFT`, `APPLIED`, `INTERVIEW`, `OFFER`, `REJECTED`, etc.), match score, and cover letter. |
| `user_preference` | User search criteria (remote only, salary range, excluded technologies, preferred locations). |
| `auto_apply_config` | Application automation settings (minimum score threshold, daily limit, selected job sources). |
| `radar_hit` & `radar_state` | Priority offer detection alerts with AI-generated justifications and notification status. |
| `cv_resume` & `cv_resume_version` | Resume repository with complete version history, metadata, and templates. |
| `offer_positioning` | Strategic positioning analysis of the candidate against specific job requirements. |
| `offer_tailored_resume` | Customized resume optimized to maximize response rate for a specific job offer. |
| `conversation` | Chat history and contextual job-search assistance sessions. |
| `users` & `authorities` | Account management, secure authentication, and access permissions. |

---

## 🛠️ Tech Stack

### Backend (Rust)
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum) (Asynchronous, built on Tokio & Tower)
- **ORM & Migrations**: [Diesel](https://diesel.rs/) (with native PostgreSQL support)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Security & Auth**: `jsonwebtoken` (JWT), `argon2`
- **API Documentation**: `utoipa` + `utoipa-scalar` (OpenAPI)
- **Observability & Logging**: `tracing`, `tracing-subscriber`

### Frontend (Client)
- **Framework**: [Angular](https://angular.dev/) (TypeScript, RxJS)
- **Architecture**: JHipster Rust Blueprint
- **Styles & UI**: Bootstrap / SCSS, Webpack
- **Internationalization**: French (`fr`), English (`en`)

### Infrastructure & Data
- **Database**: PostgreSQL 14+ (compatible with pgvector / semantic indexing)
- **Containerization**: Docker, Docker Compose
- **Monitoring**: Prometheus & Grafana (provisioning included in `docker/`)

---

## 📋 Prerequisites

Before running the project, ensure you have the following installed:

- **Rust** (`1.75+` recommended): [rustup.rs](https://rustup.rs/)
- **Node.js** (`18+` or `20+ LTS`) & **npm**
- **PostgreSQL 14+** or **Docker Desktop**
- **PostgreSQL client library (`libpq`)**:
  - *macOS*: `brew install libpq`
  - *Ubuntu/Debian*: `sudo apt-get install libpq-dev`
  - *Fedora*: `sudo dnf install libpq-devel`
  - *Windows*: Included with the official PostgreSQL installer
- **Diesel CLI**:
  ```bash
  cargo install diesel_cli --no-default-features --features postgres
  ```

---

## 🚀 Quick Start

### 1. Clone the repository and configure environment

```bash
git clone <repository-url>
cd job-search-rust
```

Create your `.env` file at root or in `server/` (see [docs/CONFIG.md](docs/CONFIG.md)):

```dotenv
DATABASE_URL=postgres://jobSearchRust:jobSearchRust@localhost:5432/jobSearchRust
SERVER_PORT=8080
JWT_SECRET=your_very_long_and_secure_jwt_secret_key
RUST_LOG=info,job_search_rust=debug
```

### 2. Start PostgreSQL Database

Start the PostgreSQL container via Docker Compose:

```bash
docker-compose up -d
# or using the npm script:
npm run docker:db:up
```

### 3. Run Diesel Migrations

Apply the complete database schema:

```bash
diesel migration run
```

### 4. Start Backend Server (Rust)

```bash
cargo run
```
The server will start at `http://localhost:8080`.

### 5. Start Frontend Application (Angular)

In a new terminal window:

```bash
npm install
npm start
```
The web interface is accessible at `http://localhost:9000` (with API proxy targeting port 8080).

---

## 🌐 Endpoints & API Documentation

The API exposes REST endpoints documented via OpenAPI / Scalar:

- 📖 **Interactive Documentation (Scalar / Swagger)**: `http://localhost:8080/scalar` or `http://localhost:8080/swagger-ui`
- 💓 **Health Check**: `GET /api/health`
- 🔑 **Authentication**: `POST /api/authenticate`
- 👤 **Account Management**: `GET /api/account`, `POST /api/register`
- 💼 **Job Offers**: `GET /api/job-offers`, `POST /api/job-offers`, `GET /api/job-offers/{id}`
- 👤 **Candidate Profiles**: `GET /api/candidate-profiles`, `POST /api/candidate-profiles`
- 📬 **Job Applications**: `GET /api/job-applications`, `POST /api/job-applications`
- 📡 **Opportunity Radar**: `GET /api/radar-hits`
- 📝 **Resumes & Versions**: `GET /api/cv-resumes`, `GET /api/cv-resume-versions`
- 🎯 **Tailored Resumes**: `GET /api/offer-tailored-resumes`

---

## ⚙️ Configuration & Environment Variables

| Variable | Description | Default Value |
| :--- | :--- | :--- |
| `DATABASE_URL` | PostgreSQL connection URL | `postgres://jobSearchRust:jobSearchRust@localhost:5432/jobSearchRust` |
| `SERVER_HOST` | HTTP server host binding | `0.0.0.0` |
| `SERVER_PORT` | Backend listening port | `8080` |
| `JWT_SECRET` | Secret key for JWT token signing | - *(Required in production)* |
| `JWT_EXPIRATION` | Token expiration time in seconds | `86400` (24h) |
| `RUST_LOG` | Tracing log verbosity level | `info,job_search_rust=debug` |

Refer to [docs/CONFIG.md](docs/CONFIG.md) for detailed configuration options.

---

## 🧪 Testing & Quality

### Unit & Integration Tests (Backend)

```bash
# Run full test suite
cargo test

# For database tests (sequential execution)
cargo test -- --test-threads=1
```

### Frontend Tests & Code Formatting

```bash
# Unit tests with Jest / Karma
npm test

# Linting & code validation
npm run lint

# Prettier format check
npm run prettier:check
```

---

## 📦 Deployment & Docker

### Build Optimized Release Binary

```bash
cargo build --release
```

### Build and Run Docker Container

```bash
# Build image
docker build -t job-search-rust .

# Run container
docker run -d -p 8080:8080 --env-file .env job-search-rust
```

---

## 📂 Project Structure

```text
job-search-rust/
├── Cargo.toml                  # Workspace manifest
├── diesel.toml                 # Diesel ORM configuration
├── docker-compose.yml          # Docker Compose stack (PostgreSQL, services)
├── Dockerfile                  # Multi-stage optimized Rust build
├── job-search-rust.jdl         # JDL domain model specification
├── migrations/                 # Database schema SQL migration scripts
│
├── client/                     # Angular frontend application
│   ├── src/
│   │   ├── app/                # Angular components, services, and models
│   │   ├── content/            # Assets, SCSS styles
│   │   └── i18n/               # Internationalization (FR / EN)
│   └── package.json
│
├── server/                     # Axum backend (Rust)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # Application entry point
│       ├── config/             # Config loader & environment handling
│       ├── db/                 # r2d2 connection pool & Diesel schema.rs
│       ├── models/             # Domain entities & Diesel models
│       ├── handlers/           # HTTP controllers / Axum routes
│       ├── services/           # Business logic & algorithms
│       ├── middleware/         # Middlewares (JWT Auth, CORS, Logger)
│       ├── errors/             # Centralized error handling (AppError)
│       └── dto/                # Data Transfer Objects
│
└── docs/                       # Technical documentation
    ├── CONFIG.md               # Configuration guide
    ├── CI_CD.md                # Continuous integration pipelines
    ├── DOCKER.md               # Containerized deployment guide
    ├── TESTING.md              # Testing guide
    └── SECURITY.md             # Security best practices
```

---

## 📄 License

This project is proprietary. All rights reserved.

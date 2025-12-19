use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Json, Redirect},
    routing::{delete, get, post},
    Router,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use oauth2::{
    basic::{
        BasicClient, BasicErrorResponse, BasicRevocationErrorResponse,
        BasicTokenIntrospectionResponse, BasicTokenResponse,
    },
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl,
    StandardRevocableToken, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

// --- Type Aliases ---

type LoftOauthClient = oauth2::Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

// --- Data Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageMetadata {
    name: String,
    version: String,
    description: Option<String>,
    manifest: serde_json::Value,
    repository: Option<String>,
    authors: Vec<String>,
    license: Option<String>,
    owners: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Package {
    metadata: PackageMetadata,
    tarball: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    github_id: u64,
    username: String,
    avatar_url: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiToken {
    id: String,
    name: String,
    user_github_id: u64,
    token_hash: String,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
struct AppState {
    packages: Arc<RwLock<HashMap<String, Vec<Package>>>>,
    users: Arc<RwLock<HashMap<u64, User>>>,
    tokens: Arc<RwLock<HashMap<String, ApiToken>>>,
    storage_dir: String,
    oauth_client: LoftOauthClient,
    jwt_secret: String,
}

impl AppState {
    fn new(
        storage_dir: String,
        client_id: String,
        client_secret: String,
        public_url: String,
    ) -> Self {
        fs::create_dir_all(&storage_dir).expect("Failed to create storage directory");

        let oauth_client = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(
                AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
            )
            .set_token_uri(
                TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap(),
            )
            .set_redirect_uri(
                RedirectUrl::new(format!("{}/auth/github/callback", public_url)).unwrap(),
            );

        Self {
            packages: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(HashMap::new())),
            storage_dir,
            oauth_client,
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string()),
        }
    }

    fn save_users(&self) {
        let users = self.users.read().unwrap();
        let file_path = format!("{}/users.json", self.storage_dir);
        let json = serde_json::to_string_pretty(&*users).unwrap();
        fs::write(file_path, json).unwrap();
    }

    fn save_tokens(&self) {
        let tokens = self.tokens.read().unwrap();
        let file_path = format!("{}/tokens.json", self.storage_dir);
        let json = serde_json::to_string_pretty(&*tokens).unwrap();
        fs::write(file_path, json).unwrap();
    }
}

// --- Request/Response Structs ---

#[derive(Deserialize)]
struct PublishRequest {
    name: String,
    version: String,
    description: Option<String>,
    manifest: serde_json::Value,
    tarball: String,
    repository: Option<String>,
    authors: Option<Vec<String>>,
    license: Option<String>,
}

#[derive(Serialize)]
struct PackageInfo {
    name: String,
    version: String,
    description: Option<String>,
    repository: Option<String>,
    authors: Vec<String>,
    license: Option<String>,
    owners: Vec<String>,
}

#[derive(Serialize)]
struct RegistryInfo {
    name: String,
    version: String,
    packages_count: usize,
    users_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    name: String,
}

#[derive(Serialize)]
struct TokenResponseStruct {
    token: String,
    name: String,
}

// --- Auth Handlers ---

async fn github_login(State(state): State<AppState>) -> Redirect {
    let (auth_url, _csrf_token) = state
        .oauth_client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("read:user".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .url();

    Redirect::to(auth_url.as_str())
}

#[derive(Deserialize)]
struct AuthCallback {
    code: String,
}

#[derive(Deserialize)]
struct GithubUser {
    id: u64,
    login: String,
    avatar_url: Option<String>,
}

async fn github_callback(
    State(state): State<AppState>,
    Query(query): Query<AuthCallback>,
) -> Result<Redirect, StatusCode> {
    println!(
        "📥 Received GitHub callback with code: {}...",
        &query.code[..8]
    );
    let client = Client::builder()
        .user_agent("loft-registry")
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token_result = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&client)
        .await
        .map_err(|e| {
            eprintln!("Token exchange error: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

    dbg!("Got token");

    let github_user: GithubUser = client
        .get("https://api.github.com/user")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    {
        let mut users = state.users.write().unwrap();
        users.insert(
            github_user.id,
            User {
                github_id: github_user.id,
                username: github_user.login.clone(),
                avatar_url: github_user.avatar_url,
                created_at: Utc::now(),
            },
        );
    }
    state.save_users();

    dbg!("Got user");

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 24 * 3600;

    let claims = Claims {
        sub: github_user.id.to_string(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    dbg!("Got token2");

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "https://loft.fargone.sh".to_string());
    Ok(Redirect::to(&format!(
        "{}/auth/callback?token={}",
        frontend_url, token
    )))
}

async fn get_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<User>, StatusCode> {
    let user_id = authenticate(&state, &headers)?;
    let users = state.users.read().unwrap();
    let user = users.get(&user_id).ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(user.clone()))
}

// --- Token Handlers ---

async fn create_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<TokenResponseStruct>, StatusCode> {
    let user_id = authenticate(&state, &headers)?;

    let token_string = Uuid::new_v4().to_string();
    let api_token = ApiToken {
        id: Uuid::new_v4().to_string(),
        name: payload.name.clone(),
        user_github_id: user_id,
        token_hash: token_string.clone(),
        created_at: Utc::now(),
        last_used_at: None,
    };

    {
        let mut tokens = state.tokens.write().unwrap();
        tokens.insert(token_string.clone(), api_token);
    }
    state.save_tokens();

    Ok(Json(TokenResponseStruct {
        token: token_string,
        name: payload.name,
    }))
}

async fn list_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ApiToken>>, StatusCode> {
    let user_id = authenticate(&state, &headers)?;
    let tokens = state.tokens.read().unwrap();
    let user_tokens: Vec<ApiToken> = tokens
        .values()
        .filter(|t| t.user_github_id == user_id)
        .cloned()
        .collect();
    Ok(Json(user_tokens))
}

async fn revoke_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let user_id = authenticate(&state, &headers)?;
    let mut tokens = state.tokens.write().unwrap();

    let token_key = tokens
        .iter()
        .find(|(_, t)| t.id == token_id && t.user_github_id == user_id)
        .map(|(k, _)| k.clone());

    if let Some(key) = token_key {
        tokens.remove(&key);
        drop(tokens);
        state.save_tokens();
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// --- Helper Functions ---

fn authenticate(state: &AppState, headers: &HeaderMap) -> Result<u64, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    if let Ok(token_data) = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        return token_data
            .claims
            .sub
            .parse()
            .map_err(|_| StatusCode::UNAUTHORIZED);
    }

    let tokens = state.tokens.read().unwrap();
    if let Some(api_token) = tokens.get(token) {
        return Ok(api_token.user_github_id);
    }

    Err(StatusCode::UNAUTHORIZED)
}

// --- Package Handlers ---

async fn get_registry_info(State(state): State<AppState>) -> Json<RegistryInfo> {
    let packages = state.packages.read().unwrap();
    let users = state.users.read().unwrap();
    Json(RegistryInfo {
        name: "loft Package Registry".to_string(),
        version: "0.1.0".to_string(),
        packages_count: packages.len(),
        users_count: users.len(),
    })
}

async fn list_packages(State(state): State<AppState>) -> Json<Vec<PackageInfo>> {
    let packages = state.packages.read().unwrap();
    let mut result = Vec::new();

    for (name, versions) in packages.iter() {
        if let Some(latest) = versions.last() {
            result.push(PackageInfo {
                name: name.clone(),
                version: latest.metadata.version.clone(),
                description: latest.metadata.description.clone(),
                repository: latest.metadata.repository.clone(),
                authors: latest.metadata.authors.clone(),
                license: latest.metadata.license.clone(),
                owners: latest.metadata.owners.clone(),
            });
        }
    }

    Json(result)
}

async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<PackageInfo>>, StatusCode> {
    let packages = state.packages.read().unwrap();

    match packages.get(&name) {
        Some(versions) => {
            let info: Vec<PackageInfo> = versions
                .iter()
                .map(|pkg| PackageInfo {
                    name: pkg.metadata.name.clone(),
                    version: pkg.metadata.version.clone(),
                    description: pkg.metadata.description.clone(),
                    repository: pkg.metadata.repository.clone(),
                    authors: pkg.metadata.authors.clone(),
                    license: pkg.metadata.license.clone(),
                    owners: pkg.metadata.owners.clone(),
                })
                .collect();
            Ok(Json(info))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn download_package(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Vec<u8>, StatusCode> {
    let packages = state.packages.read().unwrap();

    match packages.get(&name) {
        Some(versions) => {
            for pkg in versions {
                if pkg.metadata.version == version {
                    return Ok(pkg.tarball.clone());
                }
            }
            Err(StatusCode::NOT_FOUND)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn publish_package(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PublishRequest>,
) -> Result<Json<PackageInfo>, StatusCode> {
    let user_id = authenticate(&state, &headers)?;
    let username = {
        let users = state.users.read().unwrap();
        users.get(&user_id).unwrap().username.clone()
    };

    if payload.name == "std" {
        return Err(StatusCode::BAD_REQUEST);
    }

    {
        let packages = state.packages.read().unwrap();
        if let Some(versions) = packages.get(&payload.name) {
            if let Some(latest) = versions.last() {
                if !latest.metadata.owners.contains(&username) {
                    return Err(StatusCode::FORBIDDEN);
                }
            }
        }
    }

    use base64::{engine::general_purpose, Engine as _};
    let tarball = general_purpose::STANDARD
        .decode(&payload.tarball)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let package = Package {
        metadata: PackageMetadata {
            name: payload.name.clone(),
            version: payload.version.clone(),
            description: payload.description.clone(),
            manifest: payload.manifest,
            repository: payload.repository,
            authors: payload.authors.unwrap_or_default(),
            license: payload.license,
            owners: vec![username.clone()],
        },
        tarball,
    };

    let mut packages = state.packages.write().unwrap();
    packages
        .entry(payload.name.clone())
        .or_insert_with(Vec::new)
        .push(package.clone());

    let package_dir = format!("{}/{}", state.storage_dir, payload.name);
    fs::create_dir_all(&package_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let package_file = format!("{}/{}.tar.gz", package_dir, payload.version);
    fs::write(&package_file, &package.tarball).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata_file = format!("{}/{}.json", package_dir, payload.version);
    let metadata_json = serde_json::to_string_pretty(&package.metadata)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    fs::write(&metadata_file, metadata_json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract tarball to temporary directory and generate docs
    let temp_extract_dir = format!("/tmp/loft-extract-{}-{}", payload.name, payload.version);
    let _ = fs::remove_dir_all(&temp_extract_dir); // Clean up if exists
    fs::create_dir_all(&temp_extract_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract tarball
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    use tar::Archive;

    let cursor = Cursor::new(&package.tarball);
    let gz = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz);
    archive.unpack(&temp_extract_dir).map_err(|e| {
        eprintln!("Failed to extract tarball: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate documentation using loft doc command
    let docs_output = format!(
        "{}/docs/{}/{}",
        state.storage_dir, payload.name, payload.version
    );
    fs::create_dir_all(&docs_output).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Run loft doc command from extracted directory
    let doc_result = std::process::Command::new("loft")
        .arg("doc")
        .arg("-o")
        .arg(&docs_output)
        .current_dir(&temp_extract_dir)
        .output();

    match doc_result {
        Ok(output) if output.status.success() => {
            println!(
                "✓ Generated documentation for {}@{}",
                payload.name, payload.version
            );
        }
        Ok(output) => {
            eprintln!(
                "⚠ Documentation generation failed for {}@{}: {}",
                payload.name,
                payload.version,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            eprintln!("⚠ Could not run doc generator: {}", e);
        }
    }

    // Cleanup temp directory
    let _ = fs::remove_dir_all(&temp_extract_dir);

    Ok(Json(PackageInfo {
        name: package.metadata.name,
        version: package.metadata.version,
        description: package.metadata.description,
        repository: package.metadata.repository,
        authors: package.metadata.authors,
        license: package.metadata.license,
        owners: package.metadata.owners,
    }))
}

async fn get_doc_content(Path(path): Path<String>) -> Result<String, StatusCode> {
    // Prevent directory traversal
    if path.contains("..") {
        return Err(StatusCode::BAD_REQUEST);
    }
    let full_path = format!("../book/src/{}", path);
    fs::read_to_string(full_path).map_err(|_| StatusCode::NOT_FOUND)
}

async fn get_install_sh() -> Result<(HeaderMap, String), StatusCode> {
    let content = fs::read_to_string("../install.sh").map_err(|_| StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        "text/x-shellscript".parse().unwrap(),
    );
    Ok((headers, content))
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();

    let storage_dir =
        std::env::var("STORAGE_DIR").unwrap_or_else(|_| "./registry-storage".to_string());
    let client_id = std::env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
    let client_secret =
        std::env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
    let public_url =
        std::env::var("PUBLIC_URL").unwrap_or_else(|_| "https://loft.fargone.sh".to_string());

    let state = AppState::new(storage_dir, client_id, client_secret, public_url);

    if let Ok(entries) = fs::read_dir(&state.storage_dir) {
        let mut packages = state.packages.write().unwrap();

        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let package_name = entry.file_name().to_string_lossy().to_string();

                if let Ok(version_entries) = fs::read_dir(entry.path()) {
                    for version_entry in version_entries.flatten() {
                        let path = version_entry.path();

                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            if let Ok(metadata_content) = fs::read_to_string(&path) {
                                if let Ok(metadata) =
                                    serde_json::from_str::<PackageMetadata>(&metadata_content)
                                {
                                    let version = metadata.version.clone();
                                    let tarball_path =
                                        path.with_file_name(format!("{}.tar.gz", version));

                                    if let Ok(tarball) = fs::read(&tarball_path) {
                                        packages
                                            .entry(package_name.clone())
                                            .or_insert_with(Vec::new)
                                            .push(Package { metadata, tarball });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let users_file = format!("{}/users.json", state.storage_dir);
    if let Ok(content) = fs::read_to_string(&users_file) {
        if let Ok(users) = serde_json::from_str::<HashMap<u64, User>>(&content) {
            *state.users.write().unwrap() = users;
        }
    }

    let tokens_file = format!("{}/tokens.json", state.storage_dir);
    if let Ok(content) = fs::read_to_string(&tokens_file) {
        if let Ok(tokens) = serde_json::from_str::<HashMap<String, ApiToken>>(&content) {
            *state.tokens.write().unwrap() = tokens;
        }
    }

    let app = Router::new()
        .route("/", get(get_registry_info))
        .route("/install.sh", get(get_install_sh))
        .route("/packages", get(list_packages))
        .route("/packages/:name", get(get_package))
        .route("/packages/:name/:version/download", get(download_package))
        .route("/packages/publish", post(publish_package))
        .route("/auth/github/login", get(github_login))
        .route("/auth/github/callback", get(github_callback))
        .route("/auth/me", get(get_me))
        .route("/api/docs/*path", get(get_doc_content))
        .route("/tokens", post(create_token).get(list_tokens))
        .route("/tokens/:id", delete(revoke_token))
        .nest_service("/docs", ServeDir::new("../book/book"))
        .nest_service("/stdlib", ServeDir::new("../stdlib-docs"))
        .nest_service(
            "/pkg-docs",
            ServeDir::new(&format!("{}/docs", &state.storage_dir)),
        )
        .fallback_service(ServeDir::new("../www/dist"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:5050".to_string());
    println!("🚀 loft Package Registry running on http://{}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

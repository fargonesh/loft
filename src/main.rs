use loft::parser::{Parser, InputStream};
use loft::runtime::{Interpreter, value::Value, permissions::PermissionManager, permission_context};
use clap::{Parser as ClapParser, Subcommand};
use owo_colors::OwoColorize;
use miette::GraphicalReportHandler;
use rustyline::{
    error::ReadlineError,
    DefaultEditor,
};

#[derive(ClapParser)]
#[command(name = "loft")]
#[command(about = "A loft language interpreter")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Execute code directly from command line
    #[arg(short = 'c', long = "command", value_name = "CODE")]
    code: Option<String>,
    
    /// File to execute (use '.' to run from manifest.json entrypoint)
    file: Option<String>,
    
    /// Allow all permissions (file system, network, command execution)
    #[arg(long = "allow-all")]
    allow_all: bool,
    
    /// Allow read access to the file system
    #[arg(long = "allow-read")]
    allow_read: bool,
    
    /// Allow write access to the file system
    #[arg(long = "allow-write")]
    allow_write: bool,
    
    /// Allow network access
    #[arg(long = "allow-net")]
    allow_net: bool,
    
    /// Allow command execution
    #[arg(long = "allow-run")]
    allow_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run demo examples
    Demo,
    /// Create a new loft project
    New {
        /// Name of the project (use '.' for current directory)
        name: String,
    },
    /// Add a dependency to the current project
    Add {
        /// Name of the dependency
        name: String,
        /// Path or URL to the dependency (defaults to ./deps/<name>)
        #[arg(short, long)]
        path: Option<String>,
        /// Version constraint (e.g., ^1.0.0, ~2.1.0, 1.2.3)
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Update dependencies according to version constraints
    Update {
        /// Specific package to update (updates all if not specified)
        package: Option<String>,
    },
    /// Generate documentation for the current package
    Doc {
        /// Output directory for generated documentation (defaults to ./docs)
        #[arg(short, long, default_value = "docs")]
        output: String,
    },
    /// Generate standard library documentation
    StdlibDoc {
        /// Output directory for generated documentation (defaults to ./stdlib-docs)
        #[arg(short, long, default_value = "stdlib-docs")]
        output: String,
    },
    /// Format loft source files
    Format {
        /// File or directory to format (use '.' for current directory)
        path: Option<String>,
        /// Check formatting without modifying files
        #[arg(short, long)]
        check: bool,
    },
    /// Log in to the loft registry
    Login {
        /// The API token from the registry dashboard
        token: Option<String>,
    },
    /// Publish the current project to the registry
    Publish,
}

fn should_append_semicolon(input: &str) -> bool {
    let trimmed = input.trim();
    !["let", "const", "fn", "struct", "impl", "trait", "enum", "if", "while", "for", "match"]
        .iter()
        .any(|keyword| trimmed.starts_with(keyword)) 
        && !trimmed.ends_with(';')
        && !trimmed.ends_with('{')
        && !trimmed.ends_with('}')
        && !trimmed.is_empty()
}

fn main() {
    let cli = Cli::parse();
    
    // Initialize permission manager based on CLI flags
    let mut permissions = PermissionManager::with_flags(
        cli.allow_all,
        cli.allow_read,
        cli.allow_write,
        cli.allow_net,
        cli.allow_run,
    );
    
    // Load cached permissions
    let _ = permissions.load_cache();
    
    // Initialize permissions for this thread
    permission_context::init_permissions(permissions);
    
    // Priority: -c flag > file argument > subcommand > REPL
    if let Some(code) = cli.code {
        run_inline_code(&code);
    } else if let Some(file_path) = cli.file {
        // Check if file_path is "." - run from manifest.json entrypoint
        if file_path == "." {
            run_from_manifest();
        } else {
            run_file(&file_path);
        }
    } else if let Some(command) = cli.command {
        match command {
            Commands::Demo => run_demo(),
            Commands::New { name } => run_new(&name),
            Commands::Add { name, path, version } => run_add(&name, path.as_deref(), version.as_deref()),
            Commands::Update { package } => run_update(package.as_deref()),
            Commands::Doc { output } => run_doc(&output),
            Commands::StdlibDoc { output } => run_stdlib_doc(&output),
            Commands::Format { path, check } => run_format(path.as_deref(), check),
            Commands::Login { token } => run_login(token.as_deref()),
            Commands::Publish => run_publish(),
        }
    } else {
        run_repl();
    }
}

fn run_inline_code(code: &str) {
    let code_string = code.to_string();
    let stream = InputStream::new("command-line", &code_string);
    let mut parser = Parser::new(stream);
    
    match parser.parse() {
        Ok(stmts) => {
            let mut interpreter = Interpreter::with_source("command-line", &code_string);
            match interpreter.eval_program(stmts) {
                Ok(result) => {
                    if result != Value::Unit {
                        println!("{:?}", result);
                    }
                }
                Err(e) => {
                    let mut out = String::new();
                    let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                    println!("{}", out);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            let mut out = String::new();
            let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
            println!("{}", out);
            std::process::exit(1);
        }
    }
}

fn run_repl() {
    // Clear screen and show banner
    print!("\x1B[2J\x1B[1;1H");
    println!("# {} repl", "loft".bright_magenta().bold());
    println!("{}", "Type 'exit' to exit, 'help' for help".bright_black());
    println!();

    // Set up the editor
    let mut rl = DefaultEditor::new().unwrap();

    let mut interpreter = Interpreter::new();

    loop {
        let prompt = format!("{}{} ", ">".dimmed(), " ".white());
        
        match rl.readline(&prompt) {
            Ok(input) => {
                let trimmed = input.trim();
                
                // Handle special commands
                match trimmed {
                    "exit" | "quit" => {
                        println!("{}", "Goodbye!".bright_green().bold());
                        break;
                    }
                    "help" => {
                        print_help();
                        continue;
                    }
                    "clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                        continue;
                    }
                    "" => continue,
                    _ => {}
                }

                // Process the input - potentially add semicolon
                let processed_input = if should_append_semicolon(&input) {
                    format!("{};", input)
                } else {
                    input.clone()
                };

                // Add to history
                rl.add_history_entry(&input).ok();

                // Parse and evaluate
                let stream = InputStream::new("repl", &processed_input);
                let mut parser = Parser::new(stream);
                
                match parser.parse() {
                    Ok(stmts) => {
                        if stmts.is_empty() {
                            continue;
                        }
                        
                        // Create a new interpreter with source context for this line
                        // Note: In REPL mode, we need to preserve the environment between lines
                        // So we'll just use the existing interpreter and handle errors simply
                        match interpreter.eval_program(stmts) {
                            Ok(result) => {
                                // Only print non-unit values
                                if !matches!(result, Value::Unit) {
                                    println!("{}", format!("{:?}", result).bright_white());
                                }
                            }
                            Err(e) => {
                                // Use miette for pretty error printing
                                let mut out = String::new();
                                let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                                println!("{}", out);
                            }
                        }
                    }
                    Err(e) => {
                        // Use miette for pretty error printing
                        let mut out = String::new();
                        let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                        println!("{}", out);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".bright_yellow());
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "^D".bright_yellow());
                break;
            }
            Err(err) => {
                println!("{}: {:?}", "Error reading input".bright_red().bold(), err);
                break;
            }
        }
    }
}

fn print_help() {
    println!("{}", "loft REPL Help:".bright_cyan().bold());
    println!("{}", "================".bright_cyan());
    println!("{}", "Commands:".bright_yellow().bold());
    println!("  {}     - Show this help message", "help".bright_green());
    println!("  {}    - Clear the screen", "clear".bright_green());
    println!("  {}     - Exit the REPL", "exit".bright_green());
    println!();
    println!("{}", "Examples:".bright_yellow().bold());
    println!("  {}", "2 + 3 * 4".bright_white());
    println!("  {}", "let x = 42".bright_white());
    println!("  {}", "let y = x + 10; y".bright_white());
    println!("  {}", "\"Hello, world!\"".bright_white());
    println!("  {}", "term.println(\"Hello!\")".bright_white());
    println!("  {}", "let print_fn = term.print; print_fn(\"Method as value!\")".bright_white());
    println!("  {}", "[1, 2, 3]".bright_white());
    println!("  {}", "let arr = [\"a\", \"b\"]; arr.push(\"c\")".bright_white());
    println!("  {}", "arr[0]; arr.length()".bright_white());
    println!();
}

fn run_demo() {
    let examples = vec![
        ("Simple arithmetic", "2 + 3 * 4"),
        ("Variable declaration", "let x = 42;"),
        ("Expression with variables", "let x = 10; let y = 20; x + y;"),
    ];

    println!("{}", "loft Programming Language - Interpreter Demo".bright_cyan().bold());
    println!();
    println!("{}", "==============================================".bright_cyan());
    println!();

    for (name, code) in examples {
        println!("{}: {}", "Example".bright_yellow().bold(), name.bright_white().bold());
        println!("{}: {}", "Code".bright_blue().bold(), code.replace('\n', "; ").bright_white());
        
        let code_string = code.to_string();
        let stream = InputStream::new("example", &code_string);
        let mut parser = Parser::new(stream);
        
        match parser.parse() {
            Ok(stmts) => {
                let mut interpreter = Interpreter::with_source("example", &code_string);
                match interpreter.eval_program(stmts) {
                    Ok(result) => {
                        println!("{}: {}", "Result".bright_green().bold(), format!("{:?}", result).bright_white());
                    }
                    Err(e) => {
                        let mut out = String::new();
                        let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                        println!("{}", out);
                    }
                }
            }
            Err(e) => {
                let mut out = String::new();
                let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                println!("{}", out);
            }
        }
        println!();
    }
}

fn run_file(path: &str) {
    use std::fs;
    
    println!("{}: {}", "Running file".bright_cyan().bold(), path.bright_white());
    println!();
    
    match fs::read_to_string(path) {
        Ok(code) => {
            let stream = InputStream::new(path, &code);
            let mut parser = Parser::new(stream);
            
            match parser.parse() {
                Ok(stmts) => {
                    let mut interpreter = Interpreter::with_source(path, &code);
                    match interpreter.eval_program(stmts) {
                        Ok(result) => {
                            if result != Value::Unit {
                                println!();
                                println!("{}: {:?}", "Final result".bright_green().bold(), result);
                            }
                        }
                        Err(e) => {
                            println!();
                            let mut out = String::new();
                            let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                            println!("{}", out);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    let mut out = String::new();
                    let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                    println!("{}", out);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("{}: {}", "Error reading file".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn run_from_manifest() {
    use loft::manifest::Manifest;
    use std::path::Path;
    
    // Try to find and load manifest.json
    match Manifest::find_and_load(".") {
        Ok(manifest) => {
            let entrypoint_path = Path::new(&manifest.entrypoint);
            
            if !entrypoint_path.exists() {
                println!("{}: Entrypoint file '{}' not found", "Error".bright_red().bold(), manifest.entrypoint);
                std::process::exit(1);
            }
            
            println!("{}: {} ({})", "Running project".bright_cyan().bold(), 
                     manifest.name.bright_white(), 
                     manifest.entrypoint.bright_white());
            println!();
            
            // Run the entrypoint file
            run_file(&manifest.entrypoint);
        }
        Err(e) => {
            match e {
                loft::manifest::ManifestError::NotFound => {
                    println!("{}: No manifest.json found in current directory or parent directories", "Error".bright_red().bold());
                    println!("Run {} to create a new project or specify a file to run", "loft new <project-name>".bright_cyan());
                }
                _ => {
                    println!("{}: Failed to load manifest.json: {}", "Error".bright_red().bold(), e);
                }
            }
            std::process::exit(1);
        }
    }
}

fn run_new(name: &str) {
    use std::fs;
    use std::path::Path;
    
    let (project_dir, project_name) = if name == "." {
        // Create project in current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|e| {
            println!("{}: Failed to get current directory: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        });
        
        let project_name = current_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("loft-project")
            .to_string();
            
        (current_dir, project_name)
    } else {
        // Create project in new directory
        let project_dir = Path::new(name).to_path_buf();
        
        // Check if directory already exists
        if project_dir.exists() {
            println!("{}: Directory '{}' already exists", "Error".bright_red().bold(), name);
            std::process::exit(1);
        }
        
        (project_dir, name.to_string())
    };
    
    println!("{} project '{}'...", "Creating".bright_cyan().bold(), project_name.bright_white());
    
    // Create project directory if not using current directory
    if name != "." {
        if let Err(e) = fs::create_dir(&project_dir) {
            println!("{}: Failed to create directory: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
    
    // Check if manifest.json already exists
    let manifest_path = project_dir.join("manifest.json");
    if manifest_path.exists() {
        println!("{}: manifest.json already exists in this directory", "Error".bright_red().bold());
        std::process::exit(1);
    }
    
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        if let Err(e) = fs::create_dir(&src_dir) {
            println!("{}: Failed to create src directory: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
    
    // Create manifest.json
    let manifest = serde_json::json!({
        "name": project_name,
        "version": "0.1.0",
        "entrypoint": "src/main.lf",
        "dependencies": {}
    });
    
    match fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap()) {
        Ok(_) => println!("  {} manifest.json", "Created".bright_green()),
        Err(e) => {
            println!("{}: Failed to write manifest.json: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
    
    // Create src/main.lf with a simple hello world
    let main_content = r#"// Welcome to your new loft project!

term.println("Hello, world!");

// Try some basic operations
let x = 42;
let y = x * 2;
term.println("The answer is:", y);
"#;
    
    let main_path = src_dir.join("main.lf");
    if !main_path.exists() {
        match fs::write(&main_path, main_content) {
            Ok(_) => println!("  {} src/main.lf", "Created".bright_green()),
            Err(e) => {
                println!("{}: Failed to write src/main.lf: {}", "Error".bright_red().bold(), e);
                std::process::exit(1);
            }
        }
    } else {
        println!("  {} src/main.lf (already exists)", "Skipped".bright_yellow());
    }
    
    println!();
    println!("{}", "Project created successfully!".bright_green().bold());
    println!();
    if name == "." {
        println!("To get started:");
        println!("  {} .", "loft".bright_cyan());
    } else {
        println!("To get started:");
        println!("  {} {}", "cd".bright_cyan(), name);
        println!("  {} .", "loft".bright_cyan());
    }
}

fn run_add(dep_name: &str, dep_path: Option<&str>, version_constraint: Option<&str>) {
    use std::fs;
    use std::path::Path;
    use loft::manifest::Manifest;
    
    // Find manifest.json in current directory or parents
    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let manifest_path = current_dir.join("manifest.json");
    
    if !manifest_path.exists() {
        println!("{}: No manifest.json found in current directory", "Error".bright_red().bold());
        println!("Run {} to create a new project", "loft new <project-name>".bright_cyan());
        std::process::exit(1);
    }
    
    // Load existing manifest
    let mut manifest = match Manifest::load(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            println!("{}: Failed to load manifest.json: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    };
    
    // If path is provided, use local dependency, otherwise try registry
    if let Some(path) = dep_path {
        // Local dependency
        let dependency_path = path.to_string();
        
        // Check if dependency already exists
        if manifest.dependencies.contains_key(dep_name) {
            println!("{}: Dependency '{}' already exists", "Warning".bright_yellow().bold(), dep_name);
            println!("Current path: {}", manifest.dependencies.get(dep_name).unwrap());
            print!("Do you want to update it? [y/N] ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return;
            }
        }
        
        manifest.dependencies.insert(dep_name.to_string(), dependency_path.clone());
        
        // Write updated manifest
        let manifest_json = serde_json::json!({
            "name": manifest.name,
            "version": manifest.version,
            "entrypoint": manifest.entrypoint,
            "dependencies": manifest.dependencies
        });
        
        match fs::write(&manifest_path, serde_json::to_string_pretty(&manifest_json).unwrap()) {
            Ok(_) => {
                println!("{} dependency '{}' with path '{}'", 
                         "Added".bright_green().bold(), 
                         dep_name.bright_white(), 
                         dependency_path.bright_white());
            }
            Err(e) => {
                println!("{}: Failed to write manifest.json: {}", "Error".bright_red().bold(), e);
                std::process::exit(1);
            }
        }
    } else {
        // Download from registry
        println!("{} package '{}' from registry...", "Fetching".bright_cyan().bold(), dep_name.bright_white());
        
        let registry_url = std::env::var("LOFT_REGISTRY").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
        
        // Get package info
        let client = reqwest::blocking::Client::new();
        let package_url = format!("{}/packages/{}", registry_url, dep_name);
        
        let response = match client.get(&package_url).send() {
            Ok(resp) => resp,
            Err(e) => {
                println!("{}: Failed to connect to registry: {}", "Error".bright_red().bold(), e);
                println!("Make sure the registry is running at {}", registry_url);
                std::process::exit(1);
            }
        };
        
        if !response.status().is_success() {
            println!("{}: Package '{}' not found in registry", "Error".bright_red().bold(), dep_name);
            println!("Use {} to add a local dependency", "--path <path>".bright_cyan());
            std::process::exit(1);
        }
        
        let packages: Vec<serde_json::Value> = match response.json() {
            Ok(p) => p,
            Err(e) => {
                println!("{}: Failed to parse registry response: {}", "Error".bright_red().bold(), e);
                std::process::exit(1);
            }
        };
        
        if packages.is_empty() {
            println!("{}: Package '{}' has no versions", "Error".bright_red().bold(), dep_name);
            std::process::exit(1);
        }
        
        // Determine version constraint
        let constraint_str = version_constraint.unwrap_or("^0.0.0"); // Default to any version
        
        // Find best matching version
        let selected_version = if let Some(exact_version) = version_constraint.filter(|v| !v.starts_with('^') && !v.starts_with('~')) {
            // Exact version specified
            if packages.iter().any(|p| p["version"].as_str() == Some(exact_version)) {
                exact_version.to_string()
            } else {
                println!("{}: Version '{}' not found", "Error".bright_red().bold(), exact_version);
                std::process::exit(1);
            }
        } else {
            // Use constraint matching or latest
            let version_req = match semver::VersionReq::parse(constraint_str) {
                Ok(req) => req,
                Err(_) => {
                    // If parsing fails, use any version (^0.0.0 accepts all)
                    semver::VersionReq::parse(">=0.0.0").unwrap()
                }
            };
            
            // Find the best matching version
            let mut best_match: Option<String> = None;
            for pkg in &packages {
                if let Some(ver_str) = pkg["version"].as_str() {
                    if let Ok(ver) = semver::Version::parse(ver_str) {
                        if version_req.matches(&ver) {
                            best_match = Some(ver_str.to_string());
                        }
                    }
                }
            }
            
            match best_match {
                Some(v) => v,
                None => {
                    // No matching version, use latest
                    packages.last().unwrap()["version"].as_str().unwrap().to_string()
                }
            }
        };
        
        let version = &selected_version;
        
        println!("  {} version {} (constraint: {})", "Found".bright_green(), version.bright_white(), constraint_str.bright_white());
        println!("{} package...", "Downloading".bright_cyan().bold());
        
        // Download tarball
        let download_url = format!("{}/packages/{}/{}/download", registry_url, dep_name, version);
        let tarball_response = match client.get(&download_url).send() {
            Ok(resp) => resp,
            Err(e) => {
                println!("{}: Failed to download package: {}", "Error".bright_red().bold(), e);
                std::process::exit(1);
            }
        };
        
        if !tarball_response.status().is_success() {
            println!("{}: Failed to download package tarball", "Error".bright_red().bold());
            std::process::exit(1);
        }
        
        let tarball_data = match tarball_response.bytes() {
            Ok(data) => data,
            Err(e) => {
                println!("{}: Failed to read package data: {}", "Error".bright_red().bold(), e);
                std::process::exit(1);
            }
        };
        
        // Create .twlibs directory
        let twlibs_dir = current_dir.join(".twlibs");
        fs::create_dir_all(&twlibs_dir).unwrap_or_else(|e| {
            println!("{}: Failed to create .twlibs directory: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        });
        
        // Use versioned directory name
        let package_dir = twlibs_dir.join(format!("{}@{}", dep_name, version));
        if package_dir.exists() {
            fs::remove_dir_all(&package_dir).ok();
        }
        fs::create_dir_all(&package_dir).unwrap_or_else(|e| {
            println!("{}: Failed to create package directory: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        });
        
        // Extract tarball
        println!("{} package...", "Installing".bright_cyan().bold());
        
        let tar_gz = flate2::read::GzDecoder::new(&tarball_data[..]);
        let mut archive = tar::Archive::new(tar_gz);
        
        if let Err(e) = archive.unpack(&package_dir) {
            println!("{}: Failed to extract package: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
        
        // Add to dependencies with version constraint
        manifest.dependencies.insert(dep_name.to_string(), constraint_str.to_string());
        
        // Write updated manifest
        let manifest_json = serde_json::json!({
            "name": manifest.name,
            "version": manifest.version,
            "entrypoint": manifest.entrypoint,
            "dependencies": manifest.dependencies
        });
        
        match fs::write(&manifest_path, serde_json::to_string_pretty(&manifest_json).unwrap()) {
            Ok(_) => {
                println!();
                println!("{} {} v{} ({})", "Installed".bright_green().bold(), dep_name.bright_white(), version.bright_white(), constraint_str.dimmed());
                println!("  {} {}", "Location".dimmed(), package_dir.display());
            }
            Err(e) => {
                println!("{}: Failed to write manifest.json: {}", "Error".bright_red().bold(), e);
                std::process::exit(1);
            }
        }
    }
}

fn run_update(specific_package: Option<&str>) {
    use std::fs;
    use std::path::Path;
    use loft::manifest::Manifest;
    
    // Find manifest.json in current directory or parents
    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let manifest_path = current_dir.join("manifest.json");
    
    if !manifest_path.exists() {
        println!("{}: No manifest.json found in current directory", "Error".bright_red().bold());
        println!("Run {} to create a new project", "loft new <project-name>".bright_cyan());
        std::process::exit(1);
    }
    
    // Load existing manifest
    let manifest = match Manifest::load(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            println!("{}: Failed to load manifest.json: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    };
    
    let registry_url = std::env::var("LOFT_REGISTRY").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
    let client = reqwest::blocking::Client::new();
    let twlibs_dir = current_dir.join(".twlibs");
    
    // Filter dependencies to update
    let deps_to_update: Vec<(String, String)> = manifest.dependencies.iter()
        .filter(|(name, constraint)| {
            // Only update registry packages (version constraints), not local paths
            let is_local = constraint.starts_with("./") || constraint.starts_with("../") || constraint.starts_with('/');
            !is_local && (specific_package.is_none() || specific_package == Some(name.as_str()))
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    
    if deps_to_update.is_empty() {
        if let Some(pkg) = specific_package {
            println!("{}: Package '{}' not found or is a local dependency", "Error".bright_red().bold(), pkg);
        } else {
            println!("{}: No registry dependencies to update", "Info".bright_cyan().bold());
        }
        return;
    }
    
    println!("{} dependencies...", "Checking".bright_cyan().bold());
    println!();
    
    let mut updated_count = 0;
    
    for (dep_name, constraint_str) in deps_to_update {
        // Get package info from registry
        let package_url = format!("{}/packages/{}", registry_url, dep_name);
        
        let response = match client.get(&package_url).send() {
            Ok(resp) => resp,
            Err(e) => {
                println!("{}: Failed to fetch '{}': {}", "Warning".bright_yellow().bold(), dep_name, e);
                continue;
            }
        };
        
        if !response.status().is_success() {
            println!("{}: Package '{}' not found in registry", "Warning".bright_yellow().bold(), dep_name);
            continue;
        }
        
        let packages: Vec<serde_json::Value> = match response.json() {
            Ok(p) => p,
            Err(e) => {
                println!("{}: Failed to parse registry response for '{}': {}", "Warning".bright_yellow().bold(), dep_name, e);
                continue;
            }
        };
        
        if packages.is_empty() {
            println!("{}: Package '{}' has no versions", "Warning".bright_yellow().bold(), dep_name);
            continue;
        }
        
        // Parse version constraint
        let version_req = match semver::VersionReq::parse(&constraint_str) {
            Ok(req) => req,
            Err(_) => {
                println!("{}: Invalid version constraint '{}' for '{}'", "Warning".bright_yellow().bold(), constraint_str, dep_name);
                continue;
            }
        };
        
        // Find the best matching version
        let mut best_match: Option<(String, semver::Version)> = None;
        for pkg in &packages {
            if let Some(ver_str) = pkg["version"].as_str() {
                if let Ok(ver) = semver::Version::parse(ver_str) {
                    if version_req.matches(&ver)
                        && (best_match.is_none() || best_match.as_ref().unwrap().1 < ver) {
                            best_match = Some((ver_str.to_string(), ver));
                        }
                }
            }
        }
        
        let (selected_version, _) = match best_match {
            Some(v) => v,
            None => {
                println!("{}: No matching version for '{}' with constraint '{}'", "Warning".bright_yellow().bold(), dep_name, constraint_str);
                continue;
            }
        };
        
        // Check if this version is already installed
        let package_dir = twlibs_dir.join(format!("{}@{}", dep_name, selected_version));
        if package_dir.exists() {
            println!("  {} {} v{} (already up to date)", "✓".bright_green(), dep_name.bright_white(), selected_version.bright_white());
            continue;
        }
        
        // Download and install the new version
        println!("  {} {} v{} (constraint: {})", "↻".bright_cyan(), dep_name.bright_white(), selected_version.bright_white(), constraint_str.dimmed());
        
        let download_url = format!("{}/packages/{}/{}/download", registry_url, dep_name, selected_version);
        let tarball_response = match client.get(&download_url).send() {
            Ok(resp) => resp,
            Err(e) => {
                println!("    {}: Failed to download: {}", "Error".bright_red().bold(), e);
                continue;
            }
        };
        
        if !tarball_response.status().is_success() {
            println!("    {}: Failed to download tarball", "Error".bright_red().bold());
            continue;
        }
        
        let tarball_data = match tarball_response.bytes() {
            Ok(data) => data,
            Err(e) => {
                println!("    {}: Failed to read package data: {}", "Error".bright_red().bold(), e);
                continue;
            }
        };
        
        // Create package directory
        fs::create_dir_all(&twlibs_dir).ok();
        
        // Remove old versions of this package
        if let Ok(entries) = fs::read_dir(&twlibs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with(&format!("{}@", dep_name)) {
                        fs::remove_dir_all(&path).ok();
                    }
                }
            }
        }
        
        fs::create_dir_all(&package_dir).unwrap_or_else(|e| {
            println!("    {}: Failed to create package directory: {}", "Error".bright_red().bold(), e);
        });
        
        // Extract tarball
        let tar_gz = flate2::read::GzDecoder::new(&tarball_data[..]);
        let mut archive = tar::Archive::new(tar_gz);
        
        if let Err(e) = archive.unpack(&package_dir) {
            println!("    {}: Failed to extract package: {}", "Error".bright_red().bold(), e);
            fs::remove_dir_all(&package_dir).ok();
            continue;
        }
        
        println!("    {} Updated to v{}", "✓".bright_green(), selected_version.bright_white());
        updated_count += 1;
    }
    
    println!();
    if updated_count > 0 {
        println!("{} {} package(s)", "Updated".bright_green().bold(), updated_count);
    } else {
        println!("{}: All packages are up to date", "Info".bright_cyan().bold());
    }
}

fn run_doc(output_dir: &str) {
    use loft::manifest::Manifest;
    use loft::docgen::DocGenerator;
    use std::path::Path;

    println!("{}", "Generating documentation...".bright_cyan().bold());
    println!();

    // Load manifest
    let manifest = match Manifest::find_and_load(".") {
        Ok(m) => m,
        Err(e) => {
            println!("{}: {}", "Error".bright_red().bold(), e);
            println!("Make sure you're in a loft project directory with a manifest.json file.");
            std::process::exit(1);
        }
    };

    println!("Package: {}", manifest.name.bright_white().bold());
    println!("Version: {}", manifest.version.bright_white());
    println!();

    // Initialize doc generator
    let mut doc_gen = DocGenerator::new();

    // Parse the entrypoint file
    let entrypoint_path = Path::new(&manifest.entrypoint);
    if !entrypoint_path.exists() {
        println!("{}: Entrypoint file '{}' not found", "Error".bright_red().bold(), manifest.entrypoint);
        std::process::exit(1);
    }

    println!("Parsing: {}", manifest.entrypoint.bright_white());
    match doc_gen.parse_file(&manifest.entrypoint) {
        Ok(_) => {},
        Err(e) => {
            println!("{}: {}", "Parse error".bright_red().bold(), e);
            std::process::exit(1);
        }
    }

    // Find and parse all .lf files in src/
    let src_dir = Path::new("src");
    if src_dir.exists() && src_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lf") {
                    // Skip if it's already the entrypoint
                    if path != entrypoint_path {
                        println!("Parsing: {}", path.display().to_string().bright_white());
                        if let Err(e) = doc_gen.parse_file(&path) {
                            println!("{}: {}", "Warning".bright_yellow().bold(), e);
                        }
                    }
                }
            }
        }
    }

    println!();
    println!("Generating HTML...");

    // Generate HTML documentation
    let output_path = Path::new(output_dir);
    match doc_gen.generate_html(output_path, &manifest.name) {
        Ok(_) => {
            println!();
            println!("{}", "Documentation generated successfully!".bright_green().bold());
            println!();
            println!("Output directory: {}", output_path.display().to_string().bright_white());
            println!("Open {} to view the documentation", output_path.join("index.html").display().to_string().bright_cyan());
        }
        Err(e) => {
            println!();
            println!("{}: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn run_stdlib_doc(output_dir: &str) {
    use loft::docgen::stdlib::StdlibDocGenerator;
    use std::path::Path;

    println!("{}", "Generating standard library documentation...".bright_cyan().bold());
    println!();

    // Load stdlib_types.json
    let stdlib_json = include_str!("lsp/stdlib_types.json");
    
    let doc_gen = match StdlibDocGenerator::new(stdlib_json) {
        Ok(gen) => gen,
        Err(e) => {
            println!("{}: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    };

    println!("Generating HTML...");
    let output_path = Path::new(output_dir);
    match doc_gen.generate_html(output_path) {
        Ok(_) => {
            println!();
            println!("{}", "Standard library documentation generated successfully!".bright_green().bold());
            println!();
            println!("Output directory: {}", output_path.display().to_string().bright_white());
            println!("Open {} to view the documentation", output_path.join("index.html").display().to_string().bright_cyan());
        }
        Err(e) => {
            println!();
            println!("{}: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn run_login(token: Option<&str>) {
    use std::fs;
    use std::io::{self, Write};
    use std::path::PathBuf;

    let token = match token {
        Some(t) => t.to_string(),
        None => {
            print!("Enter your API token: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        }
    };

    if token.is_empty() {
        println!("{}: Token cannot be empty", "Error".bright_red().bold());
        std::process::exit(1);
    }

    let home_dir = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => {
            println!("{}: Could not find home directory", "Error".bright_red().bold());
            std::process::exit(1);
        }
    };

    let config_dir = home_dir.join(".loft");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).unwrap();
    }

    let token_file = config_dir.join("token");
    match fs::write(&token_file, token) {
        Ok(_) => {
            println!("{}", "Successfully logged in!".bright_green().bold());
            println!("Token saved to {}", token_file.display().to_string().bright_white());
        }
        Err(e) => {
            println!("{}: Failed to save token: {}", "Error".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn run_publish() {
    use loft::manifest::Manifest;
    use std::fs;
    use std::path::PathBuf;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tar::Builder;
    use base64::{engine::general_purpose, Engine as _};

    // 1. Load manifest
    let manifest = match Manifest::find_and_load(".") {
        Ok(m) => m,
        Err(_) => {
            println!("{}: No manifest.json found in current or parent directories", "Error".bright_red().bold());
            std::process::exit(1);
        }
    };

    // 2. Load token
    let home_dir = std::env::var("HOME").expect("Could not find home directory");
    let token_file = PathBuf::from(home_dir).join(".loft").join("token");
    let token = match fs::read_to_string(&token_file) {
        Ok(t) => t.trim().to_string(),
        Err(_) => {
            println!("{}: Not logged in. Run 'loft login' first.", "Error".bright_red().bold());
            std::process::exit(1);
        }
    };

    println!("📦 Publishing {}@{}...", manifest.name.bright_cyan(), manifest.version.bright_white());

    // 3. Create tarball
    let mut tar_data = Vec::new();
    {
        let enc = GzEncoder::new(&mut tar_data, Compression::default());
        let mut tar = Builder::new(enc);
        
        // Add all files in current directory (excluding .twlibs and target)
        let current_dir = std::env::current_dir().unwrap();
        for entry in fs::read_dir(current_dir).unwrap().flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            if name == ".twlibs" || name == "target" || name == ".git" {
                continue;
            }
            if path.is_dir() {
                tar.append_dir_all(name, &path).unwrap();
            } else {
                tar.append_path_with_name(&path, name).unwrap();
            }
        }
        tar.finish().unwrap();
    }

    let tarball_b64 = general_purpose::STANDARD.encode(tar_data);

    // 4. Send to registry
    let registry_url = std::env::var("LOFT_REGISTRY").unwrap_or_else(|_| "https://loft.fargone.sh".to_string());
    let client = reqwest::blocking::Client::new();
    
    #[derive(serde::Serialize)]
    struct PublishRequest {
        name: String,
        version: String,
        description: Option<String>,
        manifest: serde_json::Value,
        tarball: String,
    }

    let payload = PublishRequest {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: None, // Could be extracted from manifest if added there
        manifest: serde_json::to_value(&manifest).unwrap(),
        tarball: tarball_b64,
    };

    let response = client.post(format!("{}/packages/publish", registry_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send();

    match response {
        Ok(res) if res.status().is_success() => {
            println!("{}", "Successfully published!".bright_green().bold());
        }
        Ok(res) => {
            let status = res.status();
            let body = res.text().unwrap_or_default();
            println!("{}: Registry returned {} - {}", "Error".bright_red().bold(), status, body);
        }
        Err(e) => {
            println!("{}: Failed to connect to registry: {}", "Error".bright_red().bold(), e);
        }
    }
}

fn run_format(path: Option<&str>, check: bool) {
    use loft::formatter::TokenFormatter;
    use std::path::Path;
    use std::fs;

    let target_path = path.unwrap_or(".");
    let formatter = TokenFormatter::new();
    
    // Collect all .lf files to format
    let mut files_to_format = Vec::new();
    let path_obj = Path::new(target_path);
    
    if path_obj.is_file() {
        if path_obj.extension().and_then(|s| s.to_str()) == Some("lf") {
            files_to_format.push(path_obj.to_path_buf());
        } else {
            println!("{}: '{}' is not a .lf file", "Error".bright_red().bold(), target_path);
            std::process::exit(1);
        }
    } else if path_obj.is_dir() {
        // Find all .lf files in the directory (non-recursive for simplicity)
        match fs::read_dir(path_obj) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lf") {
                        files_to_format.push(path);
                    }
                }
            }
            Err(e) => {
                println!("{}: Failed to read directory: {}", "Error".bright_red().bold(), e);
                std::process::exit(1);
            }
        }
        
        // Also check src/ directory if it exists
        let src_dir = path_obj.join("src");
        if src_dir.exists() && src_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&src_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lf") {
                        files_to_format.push(path);
                    }
                }
            }
        }
    } else {
        println!("{}: Path '{}' does not exist", "Error".bright_red().bold(), target_path);
        std::process::exit(1);
    }
    
    if files_to_format.is_empty() {
        println!("{}: No .lf files found to format", "Warning".bright_yellow().bold());
        return;
    }
    
    let mut formatted_count = 0;
    let mut unchanged_count = 0;
    let mut error_count = 0;
    
    for file_path in &files_to_format {
        let display_path = file_path.display().to_string();
        
        // Read the file
        let original_content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                println!("{}: Failed to read '{}': {}", "Error".bright_red().bold(), display_path, e);
                error_count += 1;
                continue;
            }
        };
        
        // Format using token-based formatter (preserves comments and handles errors)
        let formatted_content = match formatter.format(&original_content) {
            Ok(content) => content,
            Err(e) => {
                println!("{}: Failed to format '{}': {}", "Error".bright_red().bold(), display_path, e);
                error_count += 1;
                continue;
            }
        };
        
        // Compare and write if changed
        if formatted_content.trim() == original_content.trim() {
            if !check {
                println!("  {} {}", "✓".dimmed(), display_path.dimmed());
            }
            unchanged_count += 1;
        } else if check {
            println!("  {} {}", "✗".bright_red(), display_path.bright_white());
            formatted_count += 1;
        } else {
            match fs::write(file_path, formatted_content) {
                Ok(_) => {
                    println!("  {} {}", "✓".bright_green(), display_path.bright_white());
                    formatted_count += 1;
                }
                Err(e) => {
                    println!("{}: Failed to write '{}': {}", "Error".bright_red().bold(), display_path, e);
                    error_count += 1;
                }
            }
        }
    }
    
    println!();
    
    if check {
        if formatted_count > 0 {
            println!("{}: {} file(s) need formatting", "Check failed".bright_red().bold(), formatted_count);
            std::process::exit(1);
        } else {
            println!("{}: All files are properly formatted", "Check passed".bright_green().bold());
        }
    } else {
        println!("{}", "Formatting complete!".bright_cyan().bold());
        if formatted_count > 0 {
            println!("  {} {} file(s) formatted", "✓".bright_green(), formatted_count);
        }
        if unchanged_count > 0 {
            println!("  {} {} file(s) already formatted", "•".dimmed(), unchanged_count);
        }
        if error_count > 0 {
            println!("  {} {} file(s) had errors", "✗".bright_red(), error_count);
            std::process::exit(1);
        }
    }
}



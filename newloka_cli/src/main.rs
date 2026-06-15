//! New Loka CLI
//!
//! Command-line interface for the New Loka health data system.
//! Supports patient management, record operations, sync, and audit queries.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "newloka")]
#[command(about = "New Loka - Local-first health data management")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Database path
    #[arg(short, long, global = true, default_value = "newloka.db")]
    db: PathBuf,

    /// Node identity name
    #[arg(short, long, global = true, default_value = "local-node")]
    node: String,

    /// Deployment tier
    #[arg(short, long, global = true, default_value = "t0")]
    tier: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new local node
    Init {
        /// Password for device master key
        #[arg(short, long)]
        password: String,
    },
    /// Create a new patient
    Patient {
        #[command(subcommand)]
        action: PatientAction,
    },
    /// Create an encounter
    Encounter {
        #[arg(short, long)]
        patient_id: String,
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Add an observation
    Observe {
        #[arg(short, long)]
        patient_id: String,
        #[arg(short, long)]
        code: String,
        #[arg(short, long)]
        value: String,
    },
    /// Search resources
    Search {
        /// Resource type
        #[arg(short, long)]
        resource_type: String,
        /// Patient ID filter
        #[arg(short, long)]
        patient_id: Option<String>,
    },
    /// Run sync with a peer
    Sync {
        /// Peer node address
        #[arg(short, long)]
        peer: String,
    },
    /// Query audit log
    Audit {
        /// Number of recent entries
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Start web server (T1+)
    Serve {
        /// Bind address
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        bind: String,
    },
}

#[derive(Subcommand)]
enum PatientAction {
    /// Create a patient
    Create {
        #[arg(short, long)]
        family: String,
        #[arg(short, long)]
        given: String,
        #[arg(short, long)]
        gender: String,
        #[arg(short, long)]
        birth_date: String,
        #[arg(short, long)]
        phone: Option<String>,
    },
    /// Get patient by ID
    Get { id: String },
    /// List all patients
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();

    match cli.command {
        Commands::Init { password } => {
            println!("Initializing New Loka node: {}", cli.node);
            let salt = newloka_core::crypto::generate_salt();
            let _dmk =
                newloka_core::crypto::DeviceMasterKey::derive_from_password(&password, &salt);
            let (identity, _signer) = newloka_core::identity::NodeIdentity::generate(
                cli.node.clone(),
                parse_tier(&cli.tier),
            );
            println!("Node ID: {}", identity.node_id);
            println!("Public Key: {:?}", identity.public_key);
            println!("Tier: {:?}", identity.tier);
            println!("Database: {}", cli.db.display());
            // Persist node identity and DMK salt to config file
            let config = serde_json::json!({
                "node_id": identity.node_id,
                "display_name": identity.display_name,
                "public_key": identity.public_key,
                "tier": identity.tier,
                "salt": salt.to_vec(),
                "created_at": chrono::Utc::now().to_rfc3339(),
            });
            let config_path = cli.db.with_extension("json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
            println!("Configuration saved to: {}", config_path.display());
        }
        Commands::Patient { action } => match action {
            PatientAction::Create {
                family,
                given,
                gender,
                birth_date,
                phone,
            } => {
                println!("Creating patient: {}, {}", family, given);
                let node_id = "local-node".to_string();
                let user_id = "cli-user".to_string();
                let meta = newloka_core::fhir::Meta::new(node_id, user_id);
                let patient = newloka_core::fhir::Patient {
                    id: uuid::Uuid::new_v4().to_string(),
                    meta,
                    identifier: vec![newloka_core::fhir::Identifier {
                        system: "newloka".to_string(),
                        value: uuid::Uuid::new_v4().to_string(),
                        use_field: Some("official".to_string()),
                    }],
                    active: true,
                    name: vec![newloka_core::fhir::HumanName {
                        use_field: Some("official".to_string()),
                        family,
                        given: vec![given],
                        prefix: vec![],
                    }],
                    telecom: if let Some(phone) = phone {
                        vec![newloka_core::fhir::ContactPoint {
                            system: "phone".to_string(),
                            value: phone,
                            use_field: Some("mobile".to_string()),
                        }]
                    } else {
                        vec![]
                    },
                    gender,
                    birth_date,
                    address: vec![],
                    marital_status: None,
                    general_practitioner: vec![],
                    managing_organization: None,
                    deceased_boolean: None,
                    deceased_date_time: None,
                };
                println!("Patient ID: {}", patient.id);
                let resource = newloka_core::fhir::FhirResource::Patient(patient);
                let json = serde_json::to_string_pretty(&resource)?;
                println!("{}", json);
            }
            PatientAction::Get { id } => {
                println!("Fetching patient: {}", id);
            }
            PatientAction::List => {
                println!("Listing patients...");
            }
        },
        Commands::Encounter { patient_id, status } => {
            println!(
                "Creating encounter for patient {} with status {:?}",
                patient_id, status
            );
        }
        Commands::Observe {
            patient_id,
            code,
            value,
        } => {
            println!(
                "Recording observation for {}: {} = {}",
                patient_id, code, value
            );
        }
        Commands::Search {
            resource_type,
            patient_id,
        } => {
            println!("Searching {} for patient {:?}", resource_type, patient_id);
        }
        Commands::Sync { peer } => {
            println!("Syncing with peer: {}", peer);
        }
        Commands::Audit { limit } => {
            println!("Querying last {} audit entries", limit);
        }
        Commands::Serve { bind } => {
            println!("Starting server on {}", bind);
            newloka_server::run(&bind).await?;
        }
    }

    Ok(())
}

fn parse_tier(s: &str) -> newloka_core::DeploymentTier {
    match s.to_lowercase().as_str() {
        "t0" | "solo" => newloka_core::DeploymentTier::T0_SoloClinician,
        "t1" | "clinic" => newloka_core::DeploymentTier::T1_SmallClinic,
        "t2" | "rural" => newloka_core::DeploymentTier::T2_RuralHospital,
        "t3" | "hospital" => newloka_core::DeploymentTier::T3_MultiDepartmentHospital,
        "t4" | "federation" => newloka_core::DeploymentTier::T4_ResearchFederation,
        _ => newloka_core::DeploymentTier::T0_SoloClinician,
    }
}

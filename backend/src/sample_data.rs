//! The sample competition used in development.
//!
//! Extracted from `database.rs`, where it accounted for 2,537 of the module's
//! 5,400 lines — nearly half a file whose actual subject is persistence. It is
//! seeded only when `SEED_SAMPLE_DATA` allows it, which production disables.

use anyhow::Result;
use sqlx::Row;

use crate::database::Database;
use crate::models::KpiScore;

pub async fn seed_sample_data(database: &Database) -> Result<()> {
    const TARGET: i64 = 10;

    let rows = sqlx::query("SELECT category, COUNT(*) as count FROM projects GROUP BY category")
        .fetch_all(&database.pool)
        .await?;
    let mut counts: std::collections::HashMap<String, i64> = rows
        .into_iter()
        .map(|r| (r.get::<String, _>("category"), r.get::<i64, _>("count")))
        .collect();

    let samples: Vec<(&str, &str, Vec<KpiScore>)> = vec![
        (
            "Smart Irrigation System",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 75.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 84.0,
                },
            ],
        ),
        (
            "Cancer Cell Detection Model",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 93.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 89.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 91.0,
                },
            ],
        ),
        (
            "Blockchain-Based Voting",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 70.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 80.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 72.0,
                },
            ],
        ),
        (
            "NLP-Based Summarization",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Prime Gap Distribution Analysis",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 82.0,
                },
            ],
        ),
        (
            "Topological Data Analysis Toolkit",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 75.0,
                },
            ],
        ),
        (
            "Quantum Dot Solar Cell Efficiency Model",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 79.0,
                },
            ],
        ),
        (
            "Low-Cost Cosmic Ray Detector",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 80.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Turkish Sign Language Recognition",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 80.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 90.0,
                },
            ],
        ),
        (
            "Crop Disease Detection via Drone Imagery",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Urban Traffic Pattern Analysis",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 89.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Public Health Dashboard for Regional Outbreaks",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Wearable ECG Anomaly Alert System",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "AI-Assisted Diabetic Retinopathy Screening",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 88.0,
                },
            ],
        ),
        (
            "Smart Water Recycling for Apartments",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 89.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Biodegradable Packaging from Agricultural Waste",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Adaptive Math Tutor for Middle Schoolers",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 82.0,
                },
            ],
        ),
        (
            "Sign Language Learning Game",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 87.0,
                },
            ],
        ),
        (
            "Autonomous Greenhouse Monitoring Rover",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 86.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 83.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 81.0,
                },
            ],
        ),
        (
            "Modular Search-and-Rescue Robot Arm",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 88.0,
                },
            ],
        ),
        (
            "Phishing Detection Browser Extension",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 79.0,
                },
            ],
        ),
        (
            "IoT Device Firmware Integrity Checker",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Real-Time Collaborative Code Editor",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Offline-First Note Taking App",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Automated API Testing Framework",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Peer-to-Peer File Sharing Client",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Voice-Controlled Task Manager",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Static Site Generator for Blogs",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Encrypted Messaging Client",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Open-Source Expense Tracker",
            "software",
            vec![
                KpiScore {
                    name: "Innovation".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Functionality".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Code Quality".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Solar-Powered Water Purifier",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Smart Traffic Light Controller",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Home Energy Usage Monitor",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Modular E-Bike Conversion Kit",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Low-Power Mesh Network for Rural Areas",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Automated Greenhouse Climate Control",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Portable Air Quality Sensor",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Smart Parking Space Finder",
            "technology",
            vec![
                KpiScore {
                    name: "Feasibility".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Sustainability".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Microplastic Detection in Freshwater Samples",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Gut Microbiome Diversity in Urban Populations",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Photocatalytic Degradation of Textile Dyes",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Machine Learning for Protein Folding Prediction",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Soil Nitrogen Fixation Rate Modeling",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Coral Bleaching Early Warning System",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Novel Antibiotic Resistance Gene Screening",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Atmospheric Aerosol Impact on Cloud Formation",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Biodegradable Enzyme-Based Water Filter",
            "science",
            vec![
                KpiScore {
                    name: "Scientific Rigor".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Impact".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Graph Coloring Algorithm for Scheduling",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Statistical Model for Epidemic Spread",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Fractal Geometry in Urban Growth Patterns",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Optimization of Traffic Flow Networks",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Game-Theoretic Model of Resource Allocation",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Number-Theoretic Cryptographic Hash Function",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Machine Learning Bias in Statistical Sampling",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Combinatorial Design for Tournament Scheduling",
            "mathematics",
            vec![
                KpiScore {
                    name: "Rigor".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Clarity".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Acoustic Levitation for Material Handling",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Piezoelectric Energy Harvesting from Footsteps",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Magnetic Levitation Train Prototype",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Thin-Film Superconductor Characterization",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Laser-Based Distance Measurement System",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Thermoelectric Generator for Waste Heat Recovery",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Optical Tweezers for Cell Manipulation",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Low-Frequency Gravitational Wave Simulation",
            "physics",
            vec![
                KpiScore {
                    name: "Theoretical Soundness".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Experimental Validation".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Originality".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Real-Time Traffic Sign Recognition",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "AI-Generated Turkish Poetry Model",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Fraud Detection in Mobile Payments",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Speech Emotion Recognition System",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Automated Resume Screening Tool",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Wildlife Species Classification from Camera Traps",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Predictive Maintenance for Industrial Equipment",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "AI-Powered Turkish Grammar Checker",
            "ai",
            vec![
                KpiScore {
                    name: "Model Performance".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Data Quality & Ethics".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "E-Commerce Customer Churn Prediction",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Social Media Sentiment During Elections",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Energy Consumption Forecasting Dashboard",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Sports Performance Analytics Platform",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Real Estate Price Trend Analysis",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Air Pollution Source Attribution Model",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Student Performance Prediction System",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Supply Chain Bottleneck Visualization",
            "data-science",
            vec![
                KpiScore {
                    name: "Analytical Depth".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Visualization & Communication".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Methodology".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Remote Physical Therapy Monitoring App",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "AI Chatbot for Mental Health Support",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Portable Ultrasound Image Enhancement",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Medication Adherence Reminder System",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Sleep Apnea Detection Wearable",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Telemedicine Triage Assistant",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Fall Detection System for Elderly",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Personalized Nutrition Recommendation Engine",
            "health-tech",
            vec![
                KpiScore {
                    name: "Clinical Applicability".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Safety & Compliance".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Vertical Farming Nutrient Optimization",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Community Solar Sharing Platform",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Plastic Waste Sorting Robot",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Rainwater Harvesting Smart Controller",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Carbon Footprint Tracking App",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Upcycled Construction Material from Waste",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Electric Vehicle Charging Load Balancer",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Reforestation Drone Seed Planter",
            "sustainability",
            vec![
                KpiScore {
                    name: "Environmental Impact".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Gamified Coding Curriculum for Kids",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "AI-Powered Essay Feedback Tool",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Virtual Science Lab Simulator",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Peer Tutoring Matchmaking Platform",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Speech Therapy Practice App",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Interactive History Timeline Builder",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Accessible Braille Learning Device",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Classroom Engagement Analytics Tool",
            "edtech",
            vec![
                KpiScore {
                    name: "Pedagogical Value".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Accessibility".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Warehouse Inventory Scanning Robot",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Bipedal Balance Control Algorithm",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Underwater Pipeline Inspection Robot",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Robotic Arm for Assistive Feeding",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Swarm Robotics for Crop Monitoring",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Autonomous Lawn Mowing Robot",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Robotic Exoskeleton for Rehabilitation",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Drone-Based Package Delivery System",
            "robotics",
            vec![
                KpiScore {
                    name: "Hardware Integration".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Autonomy".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Ransomware Behavior Detection System",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Secure Password Manager with Biometrics",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Network Intrusion Detection Dashboard",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Smart Contract Vulnerability Scanner",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Deepfake Detection Tool",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Zero-Trust Access Control Prototype",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Encrypted File Sharing for Teams",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Social Engineering Awareness Training Platform",
            "cybersecurity",
            vec![
                KpiScore {
                    name: "Security Robustness".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Feasibility".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Innovation".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Autonomous Delivery Drone Concept",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Smart Prosthetic Hand Proposal",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Flood Early Warning Network",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Campus Waste Sorting Initiative",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Assistive Reading Device for the Visually Impaired",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Rural Telehealth Kiosk",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Wildfire Detection Balloon System",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Low-Cost Water Desalination Unit",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Emergency Response Coordination App",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 79.0,
                },
            ],
        ),
        (
            "Solar-Powered Irrigation Drone",
            "odr",
            vec![
                KpiScore {
                    name: "Problem Definition".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Solution Originality".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Team Readiness".into(),
                    score: 91.0,
                },
            ],
        ),
        (
            "UAV Flight Control System — Design Review",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 78.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 85.0,
                },
            ],
        ),
        (
            "Underwater ROV Structural Design",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 80.0,
                },
            ],
        ),
        (
            "Prosthetic Hand Actuator Architecture",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 77.0,
                },
            ],
        ),
        (
            "Autonomous Ground Vehicle Sensor Fusion",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 91.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 86.0,
                },
            ],
        ),
        (
            "Satellite Payload Thermal Design",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 79.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 92.0,
                },
            ],
        ),
        (
            "Firefighting Robot Mechanical Design",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 85.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 81.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 89.0,
                },
            ],
        ),
        (
            "Exoskeleton Control System Architecture",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 73.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 92.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 78.0,
                },
            ],
        ),
        (
            "Agricultural Drone Swarm Coordination Design",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 90.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 76.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 83.0,
                },
            ],
        ),
        (
            "Search-and-Rescue Robot Power System",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 84.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 88.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 79.0,
                },
            ],
        ),
        (
            "Hybrid Rocket Engine Design Review",
            "ktr",
            vec![
                KpiScore {
                    name: "Technical Design Maturity".into(),
                    score: 87.0,
                },
                KpiScore {
                    name: "System Architecture".into(),
                    score: 82.0,
                },
                KpiScore {
                    name: "Validation Plan".into(),
                    score: 91.0,
                },
            ],
        ),
    ];

    for (name, category, kpi_scores) in samples {
        let already = *counts.get(category).unwrap_or(&0);
        if already >= TARGET {
            continue;
        }
        database
            .insert_project(
                database.default_competition_id().await?,
                None,
                name,
                category,
                kpi_scores,
                None,
                None,
            )
            .await?;
        *counts.entry(category.to_string()).or_insert(0) += 1;
    }

    Ok(())
}

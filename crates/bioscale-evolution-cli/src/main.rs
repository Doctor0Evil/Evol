use std::fs::File;
use std::path::PathBuf;
use std::time::SystemTime;

use clap::Parser;
use serde::{Deserialize, Serialize};

use bioscale_upgrade_store::{UpgradeDescriptor, HostBudget, BrainSpecs};
use bioscale_upgrade_store::aln::{AlnComplianceParticle, NeurorightsFlags};
use bioscale_metrics::schema::{MetricsSchemaVersion, MetricRegistrySnapshot};
use bioscale_metrics::validate_metrics_schema;
use bioscale_evidence::EvidenceTag;
use bioscale_envelopes::{GlobalEnvelopes, EnvelopeViolation};
use bioscale_tests_index::{UpgradeTestIndex, KaniHarnessIndex};

/// CLI for emitting daily bioscale evolution manifests and gating CI.
///
/// This binary is intended to be called by your daily script:
///   cargo run -p bioscale-evolution-cli -- --date YYYY-MM-DD
#[derive(Parser, Debug)]
#[command(name = "bioscale-evolution-cli")]
struct Args {
    /// ISO date for this evolution window, e.g. 2026-02-12
    #[arg(long)]
    date: String,

    /// Optional path override for workspace root (defaults to CWD)
    #[arg(long)]
    workspace_root: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DailyManifest {
    pub date: String,
    pub host_did: String,
    pub bostrom_address: String,
    pub git_commit: String,
    pub crate_versions: Vec<CrateVersion>,
    pub upgrades: Vec<UpgradeEntry>,
    pub bci_snapshots: BciAggregateSnapshot,
    pub aln_particles: Vec<AlnParticleEntry>,
    pub metrics_schema_version: MetricsSchemaVersion,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrateVersion {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpgradeEntry {
    pub upgrade_id: String,
    pub energy_joules: f64,
    pub protein_aa: f64,
    pub thermo_envelope: String,
    pub ml_schedule: String,
    pub reversal: ReversalSummary,
    pub evidence_hex_tags: [String; 10],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReversalSummary {
    pub allow_neuromorph_reversal: bool,
    pub explicit_reversal_order: bool,
    pub no_safer_alternative: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BciAggregateSnapshot {
    pub eeg_load_min: f32,
    pub eeg_load_max: f32,
    pub hrv_min: f32,
    pub hrv_max: f32,
    pub temp_c_min: f32,
    pub temp_c_max: f32,
    pub duty_cycle_min: f32,
    pub duty_cycle_max: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlnParticleEntry {
    pub hash: String,
    pub is_compliant: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let root = args
        .workspace_root
        .unwrap_or(std::env::current_dir()?);

    // 1. Load registered upgrades, host budget, brain specs for this date.
    let upgrades = bioscale_upgrade_store::load_upgrades_for_date(&root, &args.date)?;
    let host_budget: HostBudget =
        bioscale_upgrade_store::load_host_budget(&root)?;
    let brain_specs: BrainSpecs =
        bioscale_upgrade_store::load_brain_specs(&root)?;

    // 2. Enforce global envelope and RoH / corridor ceilings for each upgrade.
    let global_env = GlobalEnvelopes::load_from_policy(&root)?;
    for up in &upgrades {
        enforce_upgrade_safe(&global_env, &host_budget, &brain_specs, up)?;
        enforce_evidence_bundle(&up.evidence_bundle)?;
        enforce_neurorights_particle(&up.aln_particle)?;
    }

    // 3. Derive metrics schema snapshot and validate bioscale-metrics registry.
    let metrics_registry: MetricRegistrySnapshot =
        bioscale_metrics::load_registry_snapshot(&root)?;
    validate_metrics_schema(&metrics_registry)?;

    // 4. Aggregate BCI/EEG/HRV/thermal/duty snapshots for this window.
    let bci_snapshots =
        bioscale_telemetry::aggregate_bci_window(&root, &args.date)?;

    // 5. Collect ALN particles touched today.
    let aln_particles = collect_aln_particles(&upgrades);

    // 6. Enforce test and Kani coverage for critical upgrades.
    let test_index = UpgradeTestIndex::load(&root)?;
    let kani_index = KaniHarnessIndex::load(&root)?;

    for up in &upgrades {
        enforce_tests_exist(&test_index, up)?;
        if up.is_critical() {
            enforce_kani_harness_exists(&kani_index, up)?;
        }
    }

    // 7. Build manifest.
    let manifest = build_manifest(
        &root,
        &args.date,
        upgrades,
        bci_snapshots,
        aln_particles,
        metrics_registry.schema_version.clone(),
    )?;

    // 8. Write manifest and exit non-zero on any safety failure.
    let out_path = manifest_path(&root, &args.date);
    write_manifest(&out_path, &manifest)?;

    println!("bioscale-evolution-cli: wrote manifest to {:?}", out_path);
    Ok(())
}

fn enforce_upgrade_safe(
    env: &GlobalEnvelopes,
    host_budget: &HostBudget,
    brain_specs: &BrainSpecs,
    up: &UpgradeDescriptor,
) -> anyhow::Result<()> {
    // Compute predicted energy/protein/thermal/duty costs.
    let costs = env.estimate_costs(host_budget, brain_specs, up);

    // Hard fail if any corridor ceiling exceeded.
    if costs.exceeds_global_limits() {
        anyhow::bail!(
            "Upgrade {} exceeds global envelopes: {:?}",
            up.id,
            costs
        );
    }

    Ok(())
}

fn enforce_evidence_bundle(bundle: &bioscale_upgrade_store::EvidenceBundle) -> anyhow::Result<()> {
    // Required 10-tag EvidenceBundle.
    let required: [EvidenceTag; 10] = [
        EvidenceTag::A1F3C9B2,
        EvidenceTag::Tag4BE79D01,
        EvidenceTag::Tag9CD4A7E8,
        EvidenceTag::Tag2F8C6B44,
        EvidenceTag::Tag7E1DA2FF,
        EvidenceTag::Tag5B93E0C3,
        EvidenceTag::TagD0174AAC,
        EvidenceTag::Tag6AC2F9D9,
        EvidenceTag::TagC4E61B20,
        EvidenceTag::Tag8F09D5EE,
    ];

    let mut found = [false; 10];
    for tag in &bundle.tags {
        for (i, req) in required.iter().enumerate() {
            if tag == req {
                found[i] = true;
            }
        }
    }

    if found.iter().any(|f| !f) {
        anyhow::bail!(
            "Evidence bundle missing one or more required 10 tags: {:?}",
            bundle.tags
        );
    }

    if bundle.tags.len() != 10 {
        anyhow::bail!(
            "Evidence bundle must contain exactly 10 tags, found {}",
            bundle.tags.len()
        );
    }

    Ok(())
}

fn enforce_neurorights_particle(p: &AlnComplianceParticle) -> anyhow::Result<()> {
    if !p.neurorights_flags.contains(NeurorightsFlags::COGNITIVE_LIBERTY)
        || !p.neurorights_flags.contains(NeurorightsFlags::MENTAL_PRIVACY)
        || !p.neurorights_flags.contains(NeurorightsFlags::REVERSIBILITY)
        || !p.neurorights_flags.contains(NeurorightsFlags::FAIR_ACCESS)
    {
        anyhow::bail!(
            "ALNComplianceParticle for upgrade {} missing required neurorights flags",
            p.upgrade_id
        );
    }

    if !p.is_valid_hash() {
        anyhow::bail!(
            "ALNComplianceParticle hash invalid for upgrade {}",
            p.upgrade_id
        );
    }

    Ok(())
}

fn collect_aln_particles(
    upgrades: &[UpgradeDescriptor],
) -> Vec<AlnParticleEntry> {
    upgrades
        .iter()
        .map(|u| AlnParticleEntry {
            hash: u.aln_particle.hash_hex.clone(),
            is_compliant: u.aln_particle.is_compliant,
        })
        .collect()
}

fn enforce_tests_exist(
    index: &UpgradeTestIndex,
    up: &UpgradeDescriptor,
) -> anyhow::Result<()> {
    if !index.has_unit_test(&up.id) {
        anyhow::bail!(
            "Upgrade {} missing associated unit test in test index",
            up.id
        );
    }
    Ok(())
}

fn enforce_kani_harness_exists(
    index: &KaniHarnessIndex,
    up: &UpgradeDescriptor,
) -> anyhow::Result<()> {
    if !index.has_kani_harness(&up.id) {
        anyhow::bail!(
            "Critical upgrade {} missing Kani harness",
            up.id
        );
    }
    Ok(())
}

fn build_manifest(
    root: &PathBuf,
    date: &str,
    upgrades: Vec<UpgradeDescriptor>,
    bci: BciAggregateSnapshot,
    aln_particles: Vec<AlnParticleEntry>,
    metrics_schema_version: MetricsSchemaVersion,
) -> anyhow::Result<DailyManifest> {
    let git_commit = bioscale_git::current_commit(root)?;
    let crate_versions = bioscale_git::crate_versions(root)?;
    let host_did = bioscale_identity::current_host_did(root)?;
    let bostrom_address = bioscale_identity::current_bostrom_address(root)?;

    let upgrade_entries: Vec<UpgradeEntry> = upgrades
        .into_iter()
        .map(|u| UpgradeEntry {
            upgrade_id: u.id.clone(),
            energy_joules: u.energy_joules,
            protein_aa: u.protein_aa,
            thermo_envelope: u.thermo_envelope_id.clone(),
            ml_schedule: u.ml_schedule_id.clone(),
            reversal: ReversalSummary {
                allow_neuromorph_reversal: u.reversal.allow_neuromorph_reversal,
                explicit_reversal_order: u.reversal.explicit_reversal_order,
                no_safer_alternative: u.reversal.no_safer_alternative,
            },
            evidence_hex_tags: u.evidence_bundle.hex_tags(),
        })
        .collect();

    Ok(DailyManifest {
        date: date.to_string(),
        host_did,
        bostrom_address,
        git_commit,
        crate_versions,
        upgrades: upgrade_entries,
        bci_snapshots: bci,
        aln_particles,
        metrics_schema_version,
    })
}

fn manifest_path(root: &PathBuf, date: &str) -> PathBuf {
    let mut p = root.clone();
    p.push("research");
    std::fs::create_dir_all(&p).ok();
    p.push(format!("{date}-manifest.json"));
    p
}

fn write_manifest(path: &PathBuf, manifest: &DailyManifest) -> anyhow::Result<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, manifest)?;
    Ok(())
}

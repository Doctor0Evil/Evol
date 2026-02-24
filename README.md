# Evol

Evol is a transhuman-evolution system built on a biophysical-blockchain to explore, validate, and operationalize safe mutation and evolution pathways while respecting Karma and Soul boundaries for all assets and tokens. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Vision

Evol aims to provide a cryptographically-governed evolution substrate where biological, cognitive, and socio-economic changes can be proposed, simulated, and adopted in a way that is biocompatible, socially acceptable, and auditable across time. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)
The long-term goal is to make evolution-path design as rigorous and accountable as financial auditing, while remaining humane, ethical, and spiritually aware. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Core Concepts

- **Biophysical‑Blockchain**: A Rust-based ledger that encodes evolution events as biophysical transactions, enabling immutable proof-of-evolution, causal ordering, and multi-sig governance over sensitive changes. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)
- **Safe‑Mutation Pipeline**: Research-driven mutation and evolution steps are processed through schemas, policies, and shards that model risk, compatibility, and consent before any change is accepted into the canonical chain. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)
- **Karma & Soul Boundaries**: Every asset and token is evaluated against karmic and soul-aligned policy constraints, ensuring that no evolution-path violates pre-agreed ethical or spiritual limits of participating identities. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)
- **Social Acceptance Layer**: Evolution proposals are designed for human acceptance, with explicit focus on transparency, interpretability, and alignment with community norms and regulatory expectations. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Repository Layout

```text
Evol/
├─ .github/                    # Project funding, workflows and meta
├─ Evolution/
│  └─ aln/                     # ALN evolution corridors and gaze/bio mappings
├─ biophysical-blockchain/
│  └─ src/                     # Core blockchain logic and evolution mappers
├─ crates/                     # Shared Rust crates: schemas, primitives, utilities
├─ docs/                       # Concept docs, diagrams, and protocol notes
├─ host-node/
│  └─ src/                     # Host node runtime for the Evol biophysical-chain
├─ organic_cpu/
│  └─ src/                     # Organic CPU abstractions and evidence encoding
├─ policies/                   # JSON policies for tokens and evolution governance
├─ qpudatashards/              # QPU data shards, staking and ALN-based shards
├─ src/                        # Root Evol manifests and top-level orchestration
├─ LICENSE
└─ README.md
```

Key directories are briefly described below. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

### Evolution/aln

ALN (Aligned Language/Logic) corridors describe how biological, neurocognitive, or experiential signals are routed into safe evolution corridors, including gaze and biophysical mappings for controlled mutation paths. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

### biophysical-blockchain/src

Implements the evolution_reflex_sense_mapper and related primitives that bind physical/biophysical states to blockchain transactions, creating a bridge between embodied signals and on-chain governance. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

### crates

Holds reusable Rust crates such as schema definitions, evolution primitives, and shared utilities to keep the system modular and composable for higher-level services and nodes. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

### host-node/src

Defines the host node that participates in the Evol network: it validates evolution events, enforces policies, and serves as the execution surface for biophysical-blockchain operations. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

### organic_cpu/src

Encodes the concept of an “organic CPU” that can host evolutionary evidence, handle reflex loops, and represent higher-order cognitive/evolutionary processes as verifiable data structures. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

### policies

Contains machine-readable policies (for example, bostrom-evolve-token-research JSON) that define how tokens, research programs, and subject identities may evolve under ALN/KYC/DID and quantum-governance constraints. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

### qpudatashards

Includes ALN-based stake and data shard specifications that bind quantum/accelerated compute shards to specific evolution programs and staking rules. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

### src

Holds high-level Evol manifests (such as neuro-consent-operational-manifest.evo) that coordinate consent, operational boundaries, and lifecycle management for evolution-path execution. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Features

- **Transhuman Evolution Engine**  
  - Models evolution as a first-class ledger event, integrating biophysical signals, social constraints, and tokenized rights into a single execution layer. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

- **Safe‑Evolution Manifests**  
  - Each evolution-path is governed by explicit manifests (e.g., neuro-consent manifests) that encode consent, safety rules, and rollback/containment strategies. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

- **Biocompatible Governance**  
  - Policies guard against non-consensual or destabilizing mutations, enabling only evolution-steps that meet pre-defined biophysical, social, and karmic thresholds. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

- **Audit‑Ready by Design**  
  - Every accepted change is linked to immutable evidence, policies, and ALN corridors, yielding a continuous, cryptographically-verifiable trail for audits and research traceability. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Getting Started

> Note: Evol is currently in an early research and prototyping phase. Interfaces and APIs may change as the biophysical-blockchain and evolution manifests stabilize. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

1. **Clone the repository**

   ```bash
   git clone https://github.com/Doctor0Evil/Evol.git
   cd Evol
   ```

2. **Install Rust**

   Ensure you have a recent Rust toolchain (stable) installed via `rustup`. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

3. **Build the workspace**

   ```bash
   cargo build --release
   ```

4. **Run the host node (example)**

   ```bash
   cargo run -p host-node --release
   ```

5. **Explore the docs**

   - Browse the `docs/` directory for conceptual overviews and protocol notes. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)
   - Inspect `policies/` and `qpudatashards/` for examples of evolution policies and stake shards. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)
   - Review `Evolution/aln/` and `src/` to understand how ALN corridors and manifests orchestrate real evolution paths. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Use Cases

- Designing and simulating safe transhuman evolution paths for individuals, collectives, or research programs.  
- Encoding neuro-consent and biophysical evidence into a verifiable chain to support ethical experimentation and progressive enhancement.  
- Providing regulators, ethicists, and communities with an auditable substrate for evaluating proposed mutation pathways before deployment. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Governance & Ethics

Evol is explicitly designed to operate within ALN/KYC/DID-aware and quantum-cryptographic governance regimes, ensuring that every evolution event is attributable, consented, and bound by agreed ethical constraints. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)
Karma and Soul boundaries are treated as first-class policy primitives, so that spiritual, cultural, and personal commitments are never treated as afterthoughts in the evolution process. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Roadmap (High-Level)

- Formalize ALN evolution corridors and neuro-consent schemas.  
- Expand biophysical-blockchain primitives for richer sensor and evidence feeds.  
- Implement richer staking and QPU data shard logic for large-scale research programs.  
- Publish developer APIs and integration examples for external tools and agents. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## Contributing

Contributions that strengthen safety, biocompatibility, and ethical guardrails are especially welcome. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)
Please open an issue to discuss major design changes or new evolution-path modules before submitting a pull request. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

***

## License

Evol is distributed under the license found in the `LICENSE` file at the root of this repository. [github](https://github.com/SELF-Software-Evolution-Lab/AI-Epilepsy)

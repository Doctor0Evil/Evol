# Host Node Playbook

This playbook describes operational patterns for running an Evol host node.

## Startup

- Load node identity, keys, and consent manifests.
- Register available sensors and effectors.
- Connect to configured peers and gateways.

## Runtime Responsibilities

- Validate incoming sensor data and evolution proposals.
- Apply policies and manifests before accepting transitions.
- Emit evolution proposals for local changes or user‑driven requests.
- Maintain audit trails for all accepted and rejected transitions.

## Health & Telemetry

Recommended local metrics:
- Proposal rate and acceptance ratio.
- Policy violation counts.
- Latency between sensor snapshots and committed evolution steps.
- Peer connectivity status.

## Upgrade Strategy

- Treat manifest and policy changes as evolution events.
- Support staged rollouts with canary nodes.
- Provide explicit rollback paths for policies and manifests.

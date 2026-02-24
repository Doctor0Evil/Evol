# Biophysical‑Blockchain Networking in Evol

This document outlines how Evol nodes participate in a biophysical‑aware network.

## Node Roles

- Host node: Verifies evolution events, enforces policies, maintains local state.
- Sensor gateway: Bridges physical sensors to the evolution event stream.
- Effector gateway: Bridges approved evolution steps back to actuators or higher‑order systems.

## Message Types

- `SensorSnapshot`: Time‑stamped biophysical reading bundle.
- `EvolutionProposal`: Candidate state transition with evidence links.
- `EvolutionCommit`: Finalized, on‑chain evolution step.
- `PolicyUpdate`: Governance‑driven change in allowed evolution corridors.

## Transport Considerations

- Use authenticated, encrypted channels for all gateways.
- Tag messages with identity, consent‑scope, and policy‑version identifiers.
- Prefer streaming semantics for high‑frequency signals; batch where safe.

## Local‑First Behavior

Nodes should be able to:
- Buffer signals during network partitions.
- Apply local safety checks before broadcasting.
- Reconcile state after re‑joining the network using canonical chain history.

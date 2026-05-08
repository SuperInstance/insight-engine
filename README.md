# insight-engine

**Self-iterating discovery runtime.** Experiments breed experiments. Each run extracts insights that mutate into new experiments. No human in the loop.

## The Loop

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│  Hypothesis Space                                   │
│  (Eisenstein integers, constraint theory, GPU,      │
│   ternary CSP, hex lattices, SBM, topology)         │
│       │                                             │
│       ▼                                             │
│  Experiment Design ←── Mutator (novel combinations) │
│       │                        ↑                    │
│       ▼                        │                    │
│  Execute Experiment ───────────┘                    │
│       │          (results feed mutator)              │
│       ▼                                             │
│  Observation                                        │
│  (measure surprise, drift, energy, topology)        │
│       │                                             │
│       ▼                                             │
│  Insight Extraction                                 │
│  (what's anomalous? what's unexpectedly clean?)     │
│       │                                             │
│       ▼                                             │
│  ┌──────────────────────────────────┐              │
│  │ Insight quality > threshold?     │              │
│  │   YES → Promote to hypothesis    │──── new exp  │
│  │   NO  → Mutate parameters        │──── retry    │
│  └──────────────────────────────────┘              │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## What It Discovers

The runtime explores the intersection of:
- **Eisenstein integer arithmetic** (exact, zero-drift constraint checking)
- **Ternary constraint systems** ({-1, 0, +1} CSP)
- **Simulated Bifurcation** (Ising model optimization)
- **Hex lattice topology** (D₆ symmetry, disk constraints)
- **GPU parallelism** (batch evaluation, parallel AC-3)

Each experiment probes a novel combination of these domains. Surprising results breed deeper experiments. Boring results trigger parameter mutation.

## Quick Start

```bash
cargo run -- --iterations 50 --surprise-threshold 0.7
```

## Output

Each iteration emits an insight record:

```json
{
  "iteration": 23,
  "experiment": "eis_sbm_coupling",
  "parameters": {"n_vars": 64, "density": 0.3, "radius": 5},
  "observations": {
    "energy": -142.3,
    "violations": 0,
    "convergence_iters": 347,
    "surprise": 0.89
  },
  "insight": "SBM converges 2.3x faster when constraint density matches hex lattice density",
  "novelty": 0.92,
  "parent_insight": 17
}
```

## License

MIT OR Apache-2.0

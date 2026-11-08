# Research and implementation basis

## Camera-free evidence

Low-power moisture, temperature, acoustic-index, and connectivity signals can support habitat operations without retaining images. They still carry bias and privacy risk, so the implementation keeps device, zone, metric, timestamp, calibration revision, correlation, and provenance instead of presenting a raw number as ground truth.

## Robust temporal change

Median and median-absolute-deviation scoring are resistant to isolated outliers, while hysteresis prevents an alert from oscillating at one threshold. Canopy Relay implements this as per-zone/per-metric observation state. The selected Midstream boundary adds an executed DTW comparison between deterministic baseline and current anomaly sequences; it does not claim a learned seasonal model.

## Spatial and constrained-network composition

The selected WorldGraph revision provides a real serializable spatial node boundary. The selected LatentMesh revision provides real BLE-profile fragmentation and reassembly of an intervention route. These mechanisms change implementation and runtime evidence, but the prototype does not overstate them as a production GIS or live radio network.

## Bounded local inference

ruv-FANN runs a deterministic three-feature network locally. Fixed weights make exact-source runs reproducible; bounded output and an uncertainty band keep the result inspectable. This is a systems mechanism for local inference and ablation, not a trained ecological forecast.

## Governance and durability

Cedar separates authorization from model scores. OpenDAL separates durability from domain logic. A linked audit verified at restore establishes ordering and tamper detection for the demonstrated state, while denial tests prove forbidden effects do not occur.

## Falsifiable hypothesis

The combined path should execute all seven selected responsibilities and retain a durable governed result; each `without_*` arm should name the capability that makes the reduced mode non-deployable. The benchmark reports the deterministic fixture result and does not infer field efficacy, causal benefit, or statistical superiority.

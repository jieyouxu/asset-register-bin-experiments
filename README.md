# asset-register-bin-experiments

Trying to figure out Unreal's `AssetRegister.bin` file format. Target is Unreal Engine 4.27, others
we don't care about.

This code uses
<https://github.com/trumank/uasset_utils/blob/master/uasset_utils/src/asset_registry.rs>
heavily as a base reference.

## Using ser-hex to generate a trace for read events

See [trumank/ser-hex](https://github.com/trumank/ser-hex).

Example integration: <https://github.com/trumank/uesave-rs/compare/master...tracing>.

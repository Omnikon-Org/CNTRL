use crate::error::CntrlError;
use wasmtime::{Config, Engine, Module, Store, Linker, Caller};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::services::plugin::manifest::{PluginManifest, PluginPermission};

pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    pub fn new() -> Result<Self, CntrlError> {
        let mut config = Config::new();
        config.async_support(true);
        // Note: we can restrict memory and CPU time using wasmtime features here later.
        
        let engine = Engine::new(&config).map_err(|e| CntrlError::Plugin(e.to_string()))?;
        
        Ok(Self { engine })
    }

    /// Validates a WebAssembly module without executing it.
    pub fn validate_module(&self, wasm_bytes: &[u8]) -> Result<(), CntrlError> {
        Module::validate(&self.engine, wasm_bytes).map_err(|e| CntrlError::Plugin(format!("Invalid WASM module: {}", e)))
    }

    /// Loads and executes a WASM plugin from a .vibe-plugin ZIP archive in a fresh, isolated store.
    pub async fn run_plugin_zip(&self, zip_path: &Path) -> Result<String, CntrlError> {
        // 1. Read the ZIP archive
        let file = File::open(zip_path).map_err(|e| CntrlError::Plugin(format!("Could not open plugin zip: {}", e)))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| CntrlError::Plugin(format!("Invalid plugin zip: {}", e)))?;

        // 2. Read manifest.json
        let manifest: PluginManifest = {
            let mut manifest_file = archive.by_name("manifest.json").map_err(|_| CntrlError::Plugin("Missing manifest.json in plugin".to_string()))?;
            let mut manifest_str = String::new();
            manifest_file.read_to_string(&mut manifest_str).map_err(|e| CntrlError::Plugin(format!("Could not read manifest.json: {}", e)))?;
            serde_json::from_str(&manifest_str).map_err(|e| CntrlError::Plugin(format!("Invalid manifest.json: {}", e)))?
        };

        // 3. Read the entrypoint wasm module
        let mut wasm_file = archive.by_name(&manifest.entrypoint).map_err(|_| CntrlError::Plugin(format!("Missing entrypoint {} in plugin", manifest.entrypoint)))?;
        let mut wasm_bytes = Vec::new();
        wasm_file.read_to_end(&mut wasm_bytes).map_err(|e| CntrlError::Plugin(format!("Could not read wasm: {}", e)))?;

        let module = Module::new(&self.engine, &wasm_bytes).map_err(|e| CntrlError::Plugin(e.to_string()))?;
        let mut store = Store::new(&self.engine, ());
        let mut linker = Linker::new(&self.engine);
        
        // 4. Enforce permissions by selectively injecting host capabilities
        if manifest.permissions.contains(&PluginPermission::NetworkAccess) {
            linker.func_wrap("cntrl", "fetch", |_caller: Caller<'_, ()>, _ptr: i32, _len: i32| -> i32 {
                // Stub for isolated network fetch
                0
            }).map_err(|e| CntrlError::Plugin(e.to_string()))?;
        } else {
            linker.func_wrap("cntrl", "fetch", |_caller: Caller<'_, ()>, _ptr: i32, _len: i32| -> i32 {
                panic!("Plugin attempted NetworkAccess without permission")
            }).map_err(|e| CntrlError::Plugin(e.to_string()))?;
        }

        if manifest.permissions.contains(&PluginPermission::FileSystemAccess) {
            linker.func_wrap("cntrl", "read_mem_file", |_caller: Caller<'_, ()>, _ptr: i32, _len: i32| -> i32 {
                // Stub for virtualized memory-fs access
                0
            }).map_err(|e| CntrlError::Plugin(e.to_string()))?;
        } else {
            linker.func_wrap("cntrl", "read_mem_file", |_caller: Caller<'_, ()>, _ptr: i32, _len: i32| -> i32 {
                panic!("Plugin attempted FileSystemAccess without permission")
            }).map_err(|e| CntrlError::Plugin(e.to_string()))?;
        }

        if manifest.permissions.contains(&PluginPermission::IntentExecution) {
            linker.func_wrap("cntrl", "execute_intent", |_caller: Caller<'_, ()>, _ptr: i32, _len: i32| -> i32 {
                // Stub for executing intents
                0
            }).map_err(|e| CntrlError::Plugin(e.to_string()))?;
        } else {
            linker.func_wrap("cntrl", "execute_intent", |_caller: Caller<'_, ()>, _ptr: i32, _len: i32| -> i32 {
                panic!("Plugin attempted IntentExecution without permission")
            }).map_err(|e| CntrlError::Plugin(e.to_string()))?;
        }
        
        let instance = linker
            .instantiate_async(&mut store, &module)
            .await
            .map_err(|e| CntrlError::Plugin(e.to_string()))?;
            
        let run_func = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .map_err(|e| CntrlError::Plugin(format!("Missing exported 'run' function: {}", e)))?;
            
        run_func
            .call_async(&mut store, ())
            .await
            .map_err(|e| CntrlError::Plugin(format!("Plugin execution failed: {}", e)))?;
            
        Ok(format!("Plugin {} v{} executed successfully", manifest.name, manifest.version))
    }
}

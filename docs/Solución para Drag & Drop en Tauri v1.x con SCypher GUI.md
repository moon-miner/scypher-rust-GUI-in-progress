# 🎯 Solución para Drag & Drop en Tauri v1.x con SCypher GUI

## 📋 Descripción del Problema

En aplicaciones **Tauri v1.x**, el drag & drop de archivos usando eventos JavaScript estándar (`dragover`, `drop`) **NO FUNCIONA CORRECTAMENTE**. Específicamente:

- ✅ Los eventos `dragover` y `dragenter` se detectan (efecto visual funciona)
- ❌ El evento `drop` **NUNCA SE EJECUTA** - se consume a nivel del sistema
- ✅ El file browser (usando `rfd::AsyncFileDialog`) funciona perfectamente
- ❌ Drag & drop permanece completamente silencioso (sin logs de `drop`)

## 🔍 Causa Raíz del Problema

**Tauri v1.x intercepta los eventos de file drop** a nivel nativo antes de que lleguen al JavaScript. Los eventos JavaScript `drop` son **bloqueados por el sistema** y requieren usar la **API nativa de Tauri** específicamente diseñada para este propósito.

## ✅ Solución Definitiva: API Nativa de Tauri

La solución consiste en **combinar**:
1. **JavaScript** para efectos visuales (dragover, dragleave)
2. **API nativa de Tauri** para el procesamiento real del drop

## 🛠️ Implementación Paso a Paso

### **PASO 1: Configurar tauri.conf.json**

Agregar `"fileDropEnabled": true` en la configuración de ventana:

```json
{
  "tauri": {
    "windows": [
      {
        "fullscreen": false,
        "height": 800,
        "resizable": true,
        "title": "Tu App",
        "width": 1000,
        "center": true,
        "minHeight": 600,
        "minWidth": 800,
        "decorations": true,
        "transparent": false,
        "fileDropEnabled": true
      }
    ]
  }
}
```

### **PASO 2: Simplificar Event Listeners JavaScript**

**REEMPLAZAR** los eventos de drag & drop en `initializeEventListeners()`:

```javascript
// ANTES (NO FUNCIONA):
container.addEventListener('drop', async function(e) {
    // Este evento NUNCA se ejecuta en Tauri
});

// DESPUÉS (FUNCIONA):
container.addEventListener('dragover', function(e) {
    e.preventDefault();
    e.stopPropagation();
    e.dataTransfer.dropEffect = 'copy';
    e.currentTarget.classList.add('drag-over');
});

container.addEventListener('dragenter', function(e) {
    e.preventDefault();
    e.stopPropagation();
});

container.addEventListener('dragleave', function(e) {
    e.preventDefault();
    e.stopPropagation();
    e.currentTarget.classList.remove('drag-over');
});

// NO agregar addEventListener('drop') - usar API nativa
```

### **PASO 3: Agregar API Nativa de Tauri**

Agregar esta función después de `initializeTauri()`:

```javascript
// Inicializar listener nativo de Tauri para file drop
async function initializeTauriFileDrop() {
    try {
        console.log('🔄 Initializing Tauri native file drop...');

        // Importar la API de eventos de Tauri
        const { listen } = await import('https://unpkg.com/@tauri-apps/api@1/event');

        // Escuchar eventos de file drop nativos
        const unlisten = await listen('tauri://file-drop', async (event) => {
            console.log('🎯 TAURI NATIVE FILE DROP DETECTED!');
            console.log('📂 File drop event:', event);
            console.log('📂 Dropped files:', event.payload);

            // Remover efecto visual
            const container = document.getElementById('wordContainer');
            container.classList.remove('drag-over');

            try {
                // Verificar si hay contenido actual
                if (currentWords.length > 0) {
                    if (!confirm('¿Esto reemplazará las palabras actuales, continuar?')) {
                        console.log('👤 Usuario canceló reemplazo');
                        return;
                    }
                }

                const droppedFiles = event.payload;
                if (droppedFiles && droppedFiles.length > 0) {
                    const filePath = droppedFiles[0];
                    console.log('📄 Processing file:', filePath);

                    showToast('Reading dropped file...', 'info', 2000);

                    // Usar el backend de Tauri para leer el archivo
                    console.log('🔄 Calling invoke with path:', filePath);
                    const content = await invoke('read_seed_file', { path: filePath });
                    console.log('✅ File content received:', content.substring(0, 100) + '...');

                    const words = content.split(/\s+/).filter(word => word.length > 0);
                    console.log('📝 Parsed words:', words.length);

                    // Actualizar la interfaz
                    currentWords = words.map(word => word.toLowerCase());
                    editingIndex = -1;
                    renderWords();
                    updateValidationStatus();
                    updateProcessButtonState();

                    showToast(`Loaded ${words.length} words from dropped file`, 'success');
                } else {
                    console.log('❌ No files in native drop event');
                    throw new Error('No files detected in drop event');
                }
            } catch (error) {
                console.error('❌ Native file drop error:', error);
                showToast(`Failed to read dropped file: ${error}`, 'error');
            }
        });

        console.log('✅ Tauri native file drop initialized');

        // Guardar el unlisten para cleanup (opcional)
        window.tauriFileDropUnlisten = unlisten;

    } catch (error) {
        console.error('❌ Failed to initialize Tauri file drop:', error);
        console.log('🔄 File drop will use fallback method');
    }
}
```

### **PASO 4: Llamar la Función Nativa**

Al final de `initializeEventListeners()`, agregar:

```javascript
// Al final de initializeEventListeners()
initializeTauriFileDrop();
```

## 🎯 Diferencias Clave vs Navegador Web

| Aspecto | Navegador Web | Tauri v1.x |
|---------|---------------|-------------|
| **dragover** | ✅ Funciona | ✅ Funciona |
| **drop** | ✅ Funciona | ❌ Bloqueado |
| **file.path** | ❌ No disponible | ✅ Disponible |
| **API nativa** | ❌ No existe | ✅ Requerida |

## 🔍 Cómo Detectar el Problema

**Síntomas típicos:**
- Los efectos visuales de drag funciona (se pone amarillo/resaltado)
- Console logs de `dragover` aparecen
- Console logs de `drop` **NUNCA aparecen**
- No hay errores en consola
- File browser funciona perfectamente

**Logs esperados después de la solución:**
```
🔄 Initializing Tauri native file drop...
✅ Tauri native file drop initialized
🎯 TAURI NATIVE FILE DROP DETECTED!
📂 Dropped files: ["/path/to/file.txt"]
```

## 📋 Checklist de Implementación

- [ ] ✅ `tauri.conf.json` tiene `"fileDropEnabled": true`
- [ ] ✅ Eventos JavaScript simplificados (sin `addEventListener('drop')`)
- [ ] ✅ Función `initializeTauriFileDrop()` agregada
- [ ] ✅ Llamada a `initializeTauriFileDrop()` en `initializeEventListeners()`
- [ ] ✅ Import de `@tauri-apps/api/event` funciona
- [ ] ✅ Backend command `read_seed_file` existe y está registrado

## 🚨 Errores Comunes a Evitar

1. **NO usar `FileReader`** - no funciona con archivos dropeados en Tauri
2. **NO esperar que `e.dataTransfer.files` funcione** - usar `event.payload` nativo
3. **NO agregar `addEventListener('drop')`** - será ignorado silenciosamente
4. **SÍ mantener** `preventDefault()` en dragover para permitir drop
5. **SÍ usar** el mismo backend command que file browser (`read_seed_file`)

## 🎯 Resultado Final

- ✅ **Drag & drop funciona** exactamente igual que file browser
- ✅ **Efectos visuales** mantienen la UX esperada
- ✅ **Path completo** del archivo disponible directamente
- ✅ **Mismo backend** para ambos métodos (consistencia)
- ✅ **Error handling** robusto con fallbacks

---

**🔧 Esta solución es específica para Tauri v1.x y resuelve completamente el problema de drag & drop sin afectar otras funcionalidades.**

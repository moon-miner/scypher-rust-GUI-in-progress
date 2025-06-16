# SCypher Progress Bar Issue - Context Handoff

## 🎯 OBJETIVO FINAL
Implementar una **barra de progreso indeterminada** que se deslice de lado a lado (como descargas sin progreso conocido) mientras el backend procesa Argon2id, sin trabarse nunca, y al final complete visualmente al 100%.

## 📋 DESCRIPCIÓN DEL PROBLEMA

### Situación actual:
- **Aplicación**: SCypher GUI v3.0 (Tauri + Rust backend + JavaScript frontend)
- **Proceso problemático**: Transformación de seed phrase con Argon2id (2-3 segundos)
- **Problema**: Durante Argon2id, la UI se "freezea" porque bloquea el hilo principal de JavaScript

### Lo que queremos lograr:
1. **Click "Process"** → Barra aparece INMEDIATAMENTE
2. **Durante Argon2id (2-3s)** → Barra se desliza suavemente de lado a lado (efecto indeterminado)
3. **Argon2id termina** → Barra se completa al 100% y desaparece
4. **NUNCA se traba** - animación fluida todo el tiempo

## 🔧 ARQUITECTURA TÉCNICA

### Stack:
- **Frontend**: JavaScript vanilla + CSS + HTML templates
- **Backend**: Rust + Tauri
- **Proceso pesado**: `invoke('transform_seed_phrase')` - Argon2id que bloquea JS thread

### Archivos clave:
- `src/assets/js/app.js` - Función `processSeed()`
- `src/assets/css/main.css` - Estilos de progress bar
- `src/assets/templates/transform-tab.html` - HTML estructura

### HTML estructura:
```html
<div class="progress-bar" id="progressBar">
    <div class="progress-fill" id="progressFill"></div>
</div>
```

## 🧪 DEBUG Y EXPERIMENTOS REALIZADOS

### ✅ Confirmaciones:
1. **Elementos HTML existen**: `progressBar` y `progressFill` se encuentran correctamente
2. **JavaScript funciona**: Las clases se aplican (`show`, `indeterminate`)
3. **CSS se carga**: Reglas están presentes y computed styles son correctos
4. **Animación CSS funciona**: Cuando se fuerza manualmente en consola, la animación se ve perfecta

### 🎯 Debug key findings:
```javascript
// ESTOS COMANDOS EN CONSOLA FUNCIONAN PERFECTAMENTE:
const bar = document.getElementById('progressBar');
bar.classList.add('show', 'indeterminate');
// Resultado: Animación indeterminada perfecta y fluida
```

### ❌ Problemas encontrados:
1. **Timing issue**: El código automático no aplica las clases correctamente
2. **CSS conflicts**: Hay CSS duplicado que causa interferencias
3. **Argon2id blocking**: Durante `invoke()`, JavaScript se bloquea completamente

## 📝 INTENTOS DE SOLUCIÓN REALIZADOS

### Intento 1: Animación CSS indeterminada
```css
.progress-bar.indeterminate .progress-fill {
    width: 30% !important;
    background: linear-gradient(90deg, transparent 0%, #4CAF50 50%, transparent 100%);
    animation: indeterminate-slide 1.5s ease-in-out infinite;
}

@keyframes indeterminate-slide {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(400%); }
}
```
**Resultado**: Funciona manualmente, pero se traba durante `invoke()`

### Intento 2: RequestAnimationFrame keep-alive
```javascript
const keepAlive = () => {
    if (keepAnimating) {
        requestAnimationFrame(keepAlive);
    }
};
```
**Resultado**: Se sigue trabando porque `requestAnimationFrame` también se bloquea

### Intento 3: SetInterval simulation
```javascript
const simulateProgress = () => {
    currentProgress += increment;
    progressFill.style.width = `${currentProgress}%`;
};
setInterval(simulateProgress, 100);
```
**Resultado**: Progreso normal, pero no el efecto indeterminado deseado

### Intento 4: CSS con máxima especificidad
```css
div#progressBar.progress-bar.indeterminate div#progressFill.progress-fill {
    /* reglas con !important */
}
```
**Resultado**: Funciona manualmente, falla automáticamente

## 🚨 COMPORTAMIENTO ACTUAL vs DESEADO

### Actual (problemático):
1. Click → **Espera 2-3 segundos (nada visible)**
2. Aparece barra → **Se "freezea" inmediatamente**
3. **Se "desfreezea"** → Completa rápidamente

### Deseado (objetivo):
1. Click → **Barra aparece INMEDIATAMENTE**
2. **Deslizado suave y continuo** durante 2-3 segundos
3. **Completado visual** smooth al 100%

## 🔍 ANÁLISIS DEL PROBLEMA REAL

### Root cause identificado:
**Argon2id bloquea completamente el hilo principal de JavaScript**, incluyendo:
- `requestAnimationFrame`
- `setInterval` callbacks
- Actualizaciones del DOM
- Animaciones CSS (en algunos casos)

### ¿Por qué funciona en consola?
Cuando ejecutas manualmente en consola, hay tiempo suficiente entre comandos para que el CSS se aplique antes de que inicie el bloqueo.

## 💡 ENFOQUES DE SOLUCIÓN RECOMENDADOS

### Opción A: Animación CSS pura (PREFERIDA)
- Activar animación CSS antes de `invoke()`
- Asegurar que CSS tenga tiempo de establecerse
- Usar transiciones más robustas

### Opción B: Web Worker (backend change required)
- Mover Argon2id a Web Worker
- Mantener main thread libre
- **DESCARTADO** por el usuario (no quiere tocar backend)

### Opción C: Simulación híbrida
- Combinar animación CSS + JavaScript simulation
- Usar múltiples estrategias de keep-alive

## 🎯 PRÓXIMOS PASOS SUGERIDOS

### 1. Verificar CSS timing
Asegurar que la animación CSS se establezca ANTES del bloqueo:
```javascript
// Aplicar clases
progressBar.classList.add('show', 'indeterminate');

// Dar tiempo SUFICIENTE para CSS
await new Promise(resolve => setTimeout(resolve, 300));

// Recién entonces ejecutar invoke()
```

### 2. Debugging detallado
Antes de cada cambio, verificar:
- Computed styles del elemento
- Que la animación esté activa
- Timing exacto de aplicación de clases

### 3. CSS fallback robusto
Crear CSS que funcione incluso con timing imperfecto

## 📂 ARCHIVOS RELEVANTES

### JavaScript actual (app.js - función processSeed):
- Aplica clases `show` e `indeterminate`
- Ejecuta `await invoke('transform_seed_phrase')`
- Maneja completado y cleanup

### CSS actual (main.css):
- Reglas para `.progress-bar`, `.progress-bar.show`, `.progress-bar.indeterminate`
- Keyframes `indeterminate-slide`
- Especificidad alta con `!important`

### HTML (transform-tab.html):
- Estructura básica con IDs correctos
- Parte del sistema de tabs que carga dinámicamente

## 🎪 LOGS DE DEBUG ÚTILES

Los siguientes logs confirman que todo funciona técnicamente:
```
🐛 DEBUG: After adding indeterminate class: "progress-bar show indeterminate"
🐛 DEBUG: Final check - computed styles: {
    opacity: "0.999125", 
    transform: "matrix(1, 0, 0, 1, 0, -0.008747)", 
    animation: "1.5s ease-in-out infinite indeterminate-slide"
}
```

**Todo está técnicamente correcto, el problema es timing/sincronización.**

## 🎯 CONTEXTO PARA LA NUEVA IA

El usuario quiere específicamente:
- **NO tocar el backend Rust** (es aplicación de seguridad)
- **Efecto indeterminado** (deslizado lado a lado, no progreso fake)
- **Simulación frontend** que funcione durante el bloqueo de Argon2id
- **UX fluida** sin freezes visibles

La solución está muy cerca - solo necesita resolverse el timing entre aplicación de CSS y ejecución de `invoke()`.
# proy2-graficas

Raytracer en Rust desde cero para diorama voxel de campamento nocturno.

![Screenshot actual](docs/screenshots/01_encuadre_inicial.png)

![Diorama de inspiración](Reference_image.webp)

## Controles y Mapa de Teclas

### Navegación y Cámara Libre
- **W / S**: Mover punto de mira hacia adelante / atrás.
- **A / D**: Mover punto de mira hacia la izquierda / derecha.
- **Q / E**: Subir / bajar altura del punto de mira.
- **Shift (izq/der)**: Acelerar movimiento del punto de mira (2.5x).
- **Flechas Izquierda / Derecha**: Orbitar cámara horizontalmente alrededor del target.
- **Flechas Arriba / Abajo**: Orbitar cámara verticalmente (elevación).
- **Rueda del Ratón**: Zoom acercar / alejar.
- **R**: Activar / desactivar Auto-órbita continua (360° en 24 s a resolución completa).

### Modos de Render y Materiales
- **N**: Alternar mapas de normales (Normal Maps ON / OFF).
- **X**: Alternar reflexiones en la escena (Reflections ON / OFF, incluyendo agua con Fresnel).

### Presets de Cámara y Personajes (con transición suave ~0.5s)
- **1**: Encuadre inicial mirando a la fogata.
- **2**: Vista cenital / elevada general del diorama.
- **3**: Chrono (closeup con llama iluminando).
- **4**: Marle (closeup con llama iluminando).
- **5**: Lucca (closeup con llama iluminando).
- **6**: Robo (closeup con llama iluminando).
- **7**: Frog y la Masamune clavada en el suelo.
- **8**: Ayla (closeup con llama iluminando).
- **9**: Magus (closeup con llama iluminando).

### Tomas Especiales
- **F1**: Closeup de las gemas mágicas y refracción.
- **F2**: Tronco de asiento libre rasante (evaluación de mapas normales con tecla N).
- **F3**: Humo de la fogata elevándose hacia las estrellas.
- **F4**: Portal escondido entre los troncos del fondo.
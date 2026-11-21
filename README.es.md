# Aurora Lights

Mod nativo para Horizon que añade brillo dinámico a las fuentes de luz que llevas en la mano, a los
objetos luminosos caídos y a los objetos expuestos en marcos. También puede teñir esa luz y añadir un
halo violeta tenue a los objetos encantados.

Repo: https://github.com/netfeez/aura-lights-mod
Repo del shader complementario: https://github.com/netfeez/aura-bloom-shader

## Instalación

1. Construye o descarga el archivo `.hmod` de este repo.
2. Cópialo en la carpeta `mods` de tu instancia de Horizon y habilítalo.
3. Como es un mod nativo, el launcher pedirá confiar en el código antes de ejecutarlo.

El id del mod es `netfeez_aurora_lights`.

## Ajustes

Todos los ajustes aparecen en el menú de mods del juego y están en español cuando el idioma del juego
es español.

| Ajuste | Default | Qué hace |
|---|---|---|
| Otros jugadores | Activado | Muestra el brillo del objeto en mano de jugadores cercanos. |
| Objetos caídos | Activado | Muestra el brillo de los objetos luminosos caídos, incluso sin mod en el servidor. |
| Colores de luz | Desactivado | Tiñe el brillo del mod y la luz de bloque coloreada del repo de shader Aurora Bloom. |
| Brillo encantado | Activado | Añade un halo violeta a objetos y armaduras encantados, también en marcos. |
| Marcos de objetos | Activado | Muestra el brillo de los objetos luminosos expuestos en marcos. |

Activa **Colores de luz** si quieres luz coloreada en lugar de blanca.

## Compatibilidad

Este mod funciona junto al repo de shader Aurora Bloom. El shader lee la luz coloreada que emite el
mod para iluminar agua, bloques y entidades. Solo puede haber un paquete de shaders activo a la vez,
así que selecciona Aurora Bloom únicamente.

No actives también **Horizon Extras > Dynamic Lights**: ambos sistemas emitirían luces duplicadas.

El halo de encantamiento no puede cambiar el glint vanilla porque la API de shaders no expone esa
máscara. El mod genera el halo como luz coloreada y el shader lo muestra.

## Idiomas

- Inglés: `assets/netfeez_aurora_lights/lang/en_us.json`
- Español: `assets/netfeez_aurora_lights/lang/es_es.json`

Las variantes de español del juego usan el mismo archivo `es_es.json`.

## Verificación

```sh
cargo horizon-mod check
cargo horizon-mod build
```

El paquete resultante se guarda en `target/horizon-mod/netfeez_aurora_lights.hmod`.

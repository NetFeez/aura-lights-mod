# Aurora Lights

Mod nativo para Horizon que añade brillo dinámico a las fuentes de luz que llevas en la mano, a los
objetos luminosos caídos y a los objetos expuestos en marcos. También puede teñir esa luz y añadir un
halo violeta tenue a los objetos encantados.

Repo: https://github.com/netfeez/aura-lights-mod
Repo del shader complementario: https://github.com/netfeez/aura-bloom-shader

## Instalación

Descarga `netfeez_aurora_lights-0.1.1.hmod` y su `.minisig` del
[release v0.1.1](https://github.com/NetFeez/aura-lights-mod/releases/tag/v0.1.1). También puedes
añadir la URL de este repo en el campo **Add a native mod from GitHub** del launcher: los mods
nativos son el único tipo que acepta ese campo.

1. Copia el `.hmod` en la carpeta `mods` de tu instancia de Horizon y habilítalo.
2. Como es un mod nativo, el launcher pedirá confiar en el código Rust antes de compilarlo.

El id del mod es `netfeez_aurora_lights`.

### Versión de Horizon compatible

La versión 0.1.1 está fijada a Horizon `f037744a79f9982c47876f49c5a3067279f88706` y Mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`: la **Nightly del 21 de nov a las 16:57 UTC**. Si el
launcher dice «Hecho para la actualización del 21 de nov; juegas la del 20 de nov», estás en una
Nightly anterior; usa el [release v0.1.0](https://github.com/NetFeez/aura-lights-mod/releases/tag/v0.1.0),
fijado a la compilación del 20 de nov. El launcher puede compilar un mod nativo para otra revisión
cuando la API que usa no ha cambiado, pero usar la versión correspondiente evita el aviso.

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

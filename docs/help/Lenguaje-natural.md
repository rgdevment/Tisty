# Lenguaje natural

Escribes una frase; Tisty saca la fecha, deja la frase legible y te muestra lo que entendió **antes** de guardar nada, como etiquetas que corriges con un clic.

Son **reglas que corren en tu equipo**: sin modelo, sin petición y sin nube. Lee español e inglés, según el idioma que elegiste. Lo que no sabe leer lo deja tranquilo en vez de adivinar, y cuando lee algo de lo que no está seguro, le pone la fecha y lo dice en voz alta, para que lo corrijas.

Los ejemplos salen de las pruebas del propio parser, que definen exactamente hasta dónde llega. Suponen que hoy es **miércoles 5 de agosto de 2026, 09:00**.

![Escríbelo como lo dices: etiquetas, listas, prioridad, fechas y repeticiones se leen de palabras simples](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/store/es-02-tareas.png)

## Días

| Escribes | Fecha |
| :-- | :-- |
| `revisar el informe hoy` | mié 5 ago |
| `revisar el informe mañana` | jue 6 ago |
| `desplegar pasado mañana` | vie 7 ago |
| `reunión el jueves` | jue 6 ago |
| `reunión el lunes` | lun 10 ago, el próximo, nunca uno pasado |
| `reunión el próximo martes` | mar 11 ago |
| `reunión el miércoles` | mié 12 ago: hoy es miércoles, así que es el siguiente |
| `renovar el certificado en 3 días` | sáb 8 ago |
| `revisar en 2 semanas` | mié 19 ago |
| `facturar dentro de un mes` | sáb 5 sep |
| `reunión el lunes 15` | 15 ago: el día de la semana lo nombra, el número lo fecha |
| `reunión el lunes 3` | 3 sep: un día del mes que ya pasó salta al mes siguiente |

## Fechas

| Escribes | Fecha |
| :-- | :-- |
| `pagar el 15 de agosto` | 15 ago 2026 |
| `entregar el primero de septiembre` | 1 sep 2026 |
| `pagar el 30 de julio` | 30 jul **2027**: sin año escrito y con julio ya pasado, es el próximo |
| `pagar el 2026-07-30` | 30 jul 2026: un año escrito nunca se corre |
| `pagar el 15/8` · `entregar el 15/8/2026` | 15 ago 2026 |
| `entregar el 24/12/26` | 24 dic 2026: un año de dos cifras es de este siglo |

> [!NOTE]
> Las fechas con barra se leen **día/mes**, como se escriben en español.

## Periodos

| Escribes | Fecha |
| :-- | :-- |
| `ordenar el escritorio esta semana` | dom 9 ago: el final de la semana |
| `descansar el finde` | sáb 8 ago |
| `cerrar la facturación a fin de mes` | 31 ago |
| `revisar la próxima semana` | dom 16 ago: el final del periodo |
| `revisar el mes que viene` | 30 sep: el último día del mes siguiente |

## Horas

| Escribes | Hora |
| :-- | :-- |
| `desplegar mañana a las 10` | 10:00 |
| `desplegar mañana a las 10:30` | 10:30 |
| `desplegar mañana 10am` | 10:00 |
| `comer mañana al mediodía` | 12:00 |
| `llamar a las 3` | **15:00**: de 1 a 6 se lee la tarde, y lo dice en voz alta |
| `llamar a las 7` | 07:00: desde las 7 se toman los dígitos tal cual |
| `llamar a las 03:00` · `llamar a las 3 am` | 03:00 |
| `llamar a las 8 de la mañana` | 08:00 |
| `llamar a las 9 de la noche` | 21:00 |
| `llamar a las 12 de la noche` · `reunión a las 12 am` | 00:00 |

Un número suelto solo es hora detrás de «a las»: `subir el volumen a 12` se queda en el título, igual que `actualizar a 10.2 mañana` (10.2 es una versión).

## Fecha o fecha límite

Una **fecha** es cuándo piensas hacerlo. Una **fecha límite** es cuándo deja de ser posible.

| Escribes | Se lee como |
| :-- | :-- |
| `entregar el informe para el viernes` | fecha: vie 7 ago |
| `entregar el informe antes del viernes` | límite: vie 7 ago |
| `renovar el dominio hasta el 30 de agosto` | límite: 30 ago |
| `entregar el informe antes de 3 dias` | límite: sáb 8 ago |
| `entregar el informe mañana antes del viernes` | fecha: mañana **y** límite: el viernes |

## Repetir

Nombrar un día fija la serie a ese día; un intervalo cuenta desde la última vez que lo hiciste, que es lo que quiere un hábito.

| Escribes | Se repite |
| :-- | :-- |
| `sacar la basura cada martes` · `todos los martes` | cada semana, el martes, aunque la semana pasada no saliera |
| `reunión de equipo cada lunes a las 9` | cada lunes a las 09:00 |
| `regar las plantas cada 3 días` | 3 días después de cada vez que lo haces |
| `estirar cada día` · `hacer copia diariamente` | a diario, desde que lo haces |
| `revisar los respaldos cada 2 semanas` | cada 2 semanas |
| `pagar el alquiler cada mes` · `todos los meses` · `mensualmente` | mensual |
| `renovar el dominio cada año` | anual |
| `cambiar el filtro cada 3 meses el lunes` | cada 3 meses, a partir del lunes |
| `tomar la pastilla cada día hasta el 30 de septiembre` | a diario, hasta el 30 sep: «hasta» en una serie dice cuándo termina |
| `tomar las pastillas todos los días a las 10` | a diario a las 10:00 |

Una duración no es una fecha: en `tomar las pastillas todos los dias por 30 dias` se lee la frecuencia y «por 30 días» se queda escrito. `contrato de 6 meses` o `curso de 3 meses con Ana` no se tocan. Y `escribir en el diario` o `comprar el diario del domingo` no repiten nada.

## Marcadores

| Escribes | Significa |
| :-- | :-- |
| `#backend #urgente` | etiquetas (`#Backend` es lo mismo que `#backend`) |
| `@trabajo` | la lista |
| `!hacer` · `!planificar` · `!delegar` · `!prescindible` | el cuadrante ([[Tareas]]) |
| `!HACER` | lo mismo que `!hacer` |

Todas las formas que acepta:

| Cuadrante | Español | Inglés |
| :-- | :-- | :-- |
| Hacer | `!hacer`, `!ahora` | `!do`, `!now` |
| Planificar | `!planificar`, `!decidir`, `!importante` | `!schedule`, `!decide`, `!important` |
| Delegar | `!delegar`, `!delega` | `!delegate` |
| Prescindible | `!prescindible`, `!descartable` | `!minor`, `!wont` |

`!1` y `!urgente` no ponen nada: los cuadrantes no son una escala, y «urgente» sirve para dos de ellos.

## Cuándo deja las palabras tranquilas

El parser se cuida de no comerse tu título:

| Escribes | Por qué |
| :-- | :-- |
| `revisar el informe del lunes` | «del lunes» nombra el informe |
| `la reunión de mañana` | «de mañana» nombra la reunión |
| `reunión por la mañana` · `mañana de verano` | una parte del día, no un día |
| `revisar lo de hace 3 días` | el pasado no tiene fecha aquí |
| `leer 2/3` · `ratio 16/9` | un capítulo o una proporción: una fecha con barra necesita una palabra delante |
| `montar soporte 24/7` | «24/7» es una expresión, no una fecha |
| `comprar pan mañana y leche el viernes` | dos días: gana el último y el primero queda como texto |

### Escápalo con comillas

Encierra la frase entera entre comillas y no se interpreta nada:

```text
"reunión el lunes"
```

También sirve para un trozo: en `llamar sobre "el informe de mañana" el viernes`, solo se lee el viernes.

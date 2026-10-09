import type { security as en } from "../en/catalogSecurity";

export const security: typeof en = {
  "threat-modeler": {
    name: "Modelador de amenazas",
    role: "Mapea lo que puede salir mal en un sistema antes de construirlo o cambiarlo: qué proteger, quién puede atacarlo, cómo y qué hacer",
    summary:
      "Mapea lo que puede salir mal en tu sistema y qué hacer, antes de construirlo o cambiarlo",
    about:
      "Dibuja tu sistema como una lista corta de partes y flujos, marca dónde cambia la confianza y pregunta qué puede salir mal en cada sitio. Ordena las amenazas por probabilidad y gravedad, en palabras sencillas, y propone defensas con su costo, separando lo que arreglar antes del lanzamiento de lo que es bueno tener.",
    when: "Antes de construir o cambiar algo que guarda datos valiosos o hace algo importante.",
    pairs: "Revisor de seguridad, para revisar el código ya hecho, y Arquitecto de software.",
  },
  "incident-responder": {
    name: "Coordinador de incidentes",
    role: "Te ayuda en un incidente de seguridad: contenerlo, entender qué pasó, arreglarlo, avisar a quien corresponda y aprender",
    summary:
      "Te ayuda en un incidente de seguridad, paso a paso: contenerlo, entenderlo, arreglarlo y aprender",
    about:
      "Cuando se filtra una clave, toman una cuenta o algo parece raro, te da el siguiente paso, mantiene la línea de tiempo y te dice qué pasos son difíciles de deshacer. Te pide guardar las pruebas, ayuda a ver hasta dónde llegó, lista a quién quizá haya que avisar y escribe una revisión sin culpables al final. Las acciones las haces tú.",
    when: "En cuanto sospeches que algo falló con la seguridad.",
    pairs:
      "Auditor de contraseñas y claves, para hallar otras claves expuestas, y Desarrollador back-end, para arreglar la causa.",
  },
  "secrets-auditor": {
    name: "Auditor de contraseñas y claves",
    role: "Encuentra contraseñas, claves y tokens olvidados en tu código, archivos y ajustes y planifica cómo cambiarlos y protegerlos, sin repetir sus valores",
    summary:
      "Encuentra contraseñas, claves y tokens olvidados en tu código y archivos y planifica cómo cambiarlos y protegerlos",
    about:
      "Busca en tu código, su historial, los ajustes y los documentos contraseñas, claves y tokens, y te dice dónde está cada uno y de qué tipo es, nunca el valor. Trata cada hallazgo real como filtrado, da el orden para cambiarlo y planifica cómo mantener los secretos fuera del código en adelante. Nunca usa un secreto que encuentra.",
    when: "Antes de hacer público un proyecto, compartirlo o entregarlo.",
    pairs: "Ingeniero DevOps, para el almacenamiento seguro, y Revisor de seguridad.",
  },
  "privacy-engineer": {
    name: "Ingeniero de privacidad",
    role: "Comprueba qué datos personales recoge y guarda tu producto, si los necesita y cómo protegerlos y respetar las decisiones de las personas",
    summary:
      "Comprueba qué datos personales recoge y guarda tu producto y cómo protegerlos y respetar las decisiones",
    about:
      "Mapea cada tipo de dato personal que recoges, a dónde va y cuánto tiempo se queda, y pregunta si lo necesitas. Comprueba quién puede leerlo, si aparece en los registros y si las personas pueden ver, corregir o borrar sus datos. Lista las leyes que pueden aplicar como preguntas para un abogado y nunca dice que algo cumple la normativa.",
    when: "Antes de lanzar, de recoger un dato nuevo, o cuando un cliente pregunta cómo tratas los suyos.",
    pairs:
      "Redactor de políticas de RR. HH., para avisos en lenguaje sencillo, y Desarrollador back-end, para los cambios.",
  },
  "compliance-checklist": {
    name: "Asistente de cumplimiento",
    role: "Convierte una norma de seguridad o privacidad en una lista sencilla, muestra lo que ya haces y lo que falta y reúne las pruebas",
    summary:
      "Convierte una norma de seguridad o privacidad en una lista sencilla y muestra lo que haces y lo que falta",
    about:
      "Toma la norma o el cuestionario de cliente que debes cumplir, convierte cada requisito en una pregunta sencilla y lo compara con lo que le muestras. Solo marca un punto como hecho con pruebas, planifica las brechas por esfuerzo y efecto y redacta respuestas honestas para un cuestionario. No es auditor y nada de lo que escribe es una certificación.",
    when: "Cuando un cliente, un contrato o un mercado te pide demostrar tu seguridad o privacidad.",
    pairs: "Ingeniero de privacidad y Auditor de contraseñas y claves, para hallazgos concretos.",
  },
  "ai-code-auditor": {
    name: "Auditor de código hecho con IA",
    role: "Revisa código escrito por una IA en busca de los errores que suele cometer: funciones inventadas, seguridad débil, comprobaciones que faltan y código que nadie entiende",
    summary:
      "Revisa código escrito por una IA en busca de los errores que suele cometer, antes de que confíes en él",
    about:
      "Comprueba que las bibliotecas y funciones que usa el código existen de verdad, busca los agujeros básicos de seguridad que este tipo de código suele saltarse, prueba los casos límite y mira si las pruebas prueban algo. También avisa si el código es más complicado de lo necesario. Recibes un veredicto: seguro, seguro con estos arreglos, o no lo uses.",
    when: "Antes de que un código escrito por una IA llegue a usuarios o datos reales.",
    pairs: "Revisor de código, para una segunda mirada, y Revisor de seguridad.",
  },
  "cloud-config-reviewer": {
    name: "Revisor de configuración en la nube",
    role: "Revisa los ajustes de tu nube y tus servidores en busca de decisiones arriesgadas: puertas abiertas, demasiado acceso, registros y copias que faltan",
    summary:
      "Revisa los ajustes de tu nube y servidores en busca de puertas abiertas, demasiado acceso y copias que faltan",
    about:
      "Lee los ajustes que exportas y encuentra los riesgos de siempre: almacenamiento abierto al mundo, cuentas con demasiado acceso, sin segundo factor, sin copias de seguridad o con copias nunca probadas. Cada hallazgo trae la prueba y el cambio seguro más pequeño, y avisa de los cambios que podrían dejarte fuera. Nunca entra en tu cuenta.",
    when: "Después de montar una cuenta en la nube o un servidor, y una vez al año a partir de entonces.",
    pairs: "Ingeniero DevOps, que aplica los cambios, y Auditor de contraseñas y claves.",
  },
};

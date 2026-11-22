import type { Messages } from "../en";

export const server: Messages["server"] = {
  connecting: (name) => `Conectando a ${name}…`,
  address: "tu@tu-servidor",
  copy: "Copiar",
  copied: "Copiado",
  sending: (name) => `Enviando ${name} al servidor…`,
  sendFailed: "No se pudo enviar el archivo",
  folder: "Carpeta en el servidor",
  folderHint:
    "Una carpeta del servidor, por su ruta allí. Vacío, una carpeta nueva dentro de Botloft. Para código, pide a la crew que clone el repositorio en su carpeta.",
  useThisComputer: "Usar esta computadora",
  unreachable: {
    title: (name) => `No encuentro ${name}`,
    body: "Esta app llega al servidor por un túnel que empieza en esta computadora. Ábrelo en una terminal y déjalo abierto:",
    retrying: "Botloft sigue intentando y se conecta en cuanto el túnel se abra.",
  },
  refused: {
    title: (name) => `${name} no aceptó esta computadora`,
    body: "Puede que su llave se haya quitado allí. En el servidor, ejecuta sudo botloftd pair y pega la llave nueva, o vuelve a esta computadora.",
  },
  form: {
    title: "Conectar a un servidor",
    intro:
      "Ejecuta Botloft en un servidor tuyo y los agentes siguen trabajando con esta computadora apagada. En el servidor, ejecuta sudo botloftd pair y pega la llave que muestre.",
    key: "Llave de conexión",
    show: "Mostrar la llave",
    hide: "Ocultar la llave",
    placeholder: "botloft1.…",
    invalid: "Eso no es una llave de Botloft. Copia la línea entera que empieza con botloft1.",
    found: (name) => `Servidor: ${name}`,
    tunnel: "Esta computadora llega a él por un túnel. Ábrelo en una terminal y déjalo abierto:",
    connect: "Conectar",
  },
  where: {
    title: "Dónde corren los agentes",
    thisComputer: "En esta computadora",
    thisComputerHint: "Botloft corre aquí, en segundo plano.",
    onServer: (name) => `En el servidor ${name}`,
    onServerHint:
      "Esta app muestra los agentes que corren en el servidor. Siguen trabajando con esta computadora apagada.",
    toServer: "Conectar a un servidor",
    toThisComputer: "Usar esta computadora",
    toThisComputerTitle: "¿Usar el Botloft de esta computadora?",
    toThisComputerText:
      "Los agentes del servidor siguen corriendo. Esta app pasa a mostrar los de esta computadora, y puedes conectarte al servidor otra vez con la misma llave.",
  },
  devices: {
    title: "Dispositivos",
    hint: "Computadoras que llegan a estos agentes con una llave propia.",
    none: "Ningún otro dispositivo tiene llave.",
    thisOne: "Esta computadora",
    seen: (time) => `Se conectó por última vez ${time}`,
    never: "Nunca se conectó",
    remove: "Quitar",
    removeTitle: (name) => `¿Quitar ${name}?`,
    removeText: "Su llave deja de valer al instante y se desconecta.",
    failed: "No se pudo quitar",
  },
};

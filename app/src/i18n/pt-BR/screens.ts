// A área de design (spec 22.5): as telas HTML que o bot faz, ao vivo ao
// lado do chat enquanto ele as escreve.

import type { Messages } from "../en";

export const screens: Messages["screens"] = {
  heading: "Telas",
  panel: (bot) => `Telas de ${bot}`,
  show: "Mostrar telas",
  hide: "Ocultar telas",
  drawing: (bot) => `${bot} está desenhando uma tela`,
  showInPanel: "Ver em telas",
  close: "Fechar",
  expand: "Alargar",
  shrink: "Estreitar",
  zoomIn: "Aproximar",
  zoomOut: "Afastar",
  fit: "Ajustar",
  board: "Telas",
  open: (name) => `Abrir ${name}`,
  writing: "Escrevendo…",
  back: "Todas as telas",
  openFile: "Abrir",
  reveal: "Mostrar na pasta",
  device: "Aparelho",
  devices: {
    desktop: "Computador",
    tablet: "Tablet",
    mobile: "Celular",
  },
  more: (count) => `Mostrar mais ${count}`,
  emptyTitle: (bot) => `${bot} ainda não fez nenhuma tela`,
  emptyBody:
    "Cada página HTML que ele escrever aparece aqui, e você a vê tomar forma enquanto ele escreve.",
  loadFailed: "Não foi possível carregar as telas",
  gone: "Esta tela não está mais lá.",
  failed: {
    open: "Não foi possível abrir a tela",
    reveal: "Não foi possível mostrar o arquivo",
  },
};

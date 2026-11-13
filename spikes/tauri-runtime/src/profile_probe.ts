// A fixed synthetic measurement of the real loaded document, not emulated DPI.
// No user agent, file name, URL, patient value or arbitrary text crosses IPC.
export interface ProfileReport {
  scale_milli: number;
  body_font_milli: number;
  title_font_milli: number;
  normal_face_loaded: boolean;
  semibold_face_loaded: boolean;
  body_uses_plex: boolean;
  title_uses_plex: boolean;
  no_horizontal_overflow: boolean;
  title_inside_viewport: boolean;
}

export function measureProfile(): ProfileReport {
  const title = document.getElementById("title");
  if (title === null) throw new Error("Missing synthetic profile heading");
  const faces = Array.from(document.fonts);
  const loaded = (weights: string[]) => faces.some((face) =>
    face.family.replaceAll('"', "") === "IBM Plex Sans" && weights.includes(face.weight) && face.status === "loaded");
  const bodyStyle = getComputedStyle(document.body);
  const titleStyle = getComputedStyle(title);
  const bounds = title.getBoundingClientRect();
  return {
    scale_milli: Math.round(devicePixelRatio * 1000),
    body_font_milli: Math.round(Number.parseFloat(bodyStyle.fontSize) * 1000),
    title_font_milli: Math.round(Number.parseFloat(titleStyle.fontSize) * 1000),
    normal_face_loaded: loaded(["normal", "400"]),
    semibold_face_loaded: loaded(["600"]),
    body_uses_plex: bodyStyle.fontFamily.startsWith('"IBM Plex Sans"') || bodyStyle.fontFamily.startsWith("IBM Plex Sans"),
    title_uses_plex: titleStyle.fontFamily.startsWith('"IBM Plex Sans"') || titleStyle.fontFamily.startsWith("IBM Plex Sans"),
    no_horizontal_overflow: document.documentElement.scrollWidth <= innerWidth,
    title_inside_viewport: bounds.left >= 0 && bounds.top >= 0 && bounds.right <= innerWidth && bounds.bottom <= innerHeight,
  };
}

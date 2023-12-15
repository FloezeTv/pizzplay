import React, { Fragment, useState } from 'react'
import ReactDOM from 'react-dom/client'
import styles from './display.module.css'
import './index.css'
import { useEventListener } from './eventlistener'
import { classList } from './utils'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Display />
  </React.StrictMode>,
)

type Image = {
  url: string;
  title: string;
  subtitle: string;
};

const isImage = (o: unknown): o is Image => {
  return typeof o === "object" && o !== null && "url" in o && typeof o.url === "string" && "title" in o && typeof o.title === "string" && "subtitle" in o && typeof o.subtitle === "string";
};

// For customer-facing displays
export default function Display() {
  const [images, setImages] = useState<[Image, number][]>([]);
  const [popup, setPopup] = useState<{ text: string, show: boolean }>({ text: '', show: false });

  /**
   * Adds a new image to the display
   * @param img image to add
   */
  const addImage = async (img: Image) => {
    // Hack to prefetch image
    const i = new Image();
    i.src = img.url;
    await new Promise((resolve) => i.onload = resolve);

    setImages(i => {
      // Use incrementing number as key
      const last = i[i.length - 1];
      const idx = (last !== undefined ? last[1] : -1) + 1;
      const n = i.slice(Math.max(i.length - 2, 0));
      n.push([img, idx]);
      return n;
    })
  };

  // The event listeners for the server
  useEventListener(`${import.meta.env.DEV ? /* sse-proxy is kind of broken */ 'http://localhost:8080' : import.meta.env.BASE_URL}/events/subscribe?image_change&popup_show&popup_hide`, {
    'image_change': (d) => {
      try {
        const json = JSON.parse(d);
        if (isImage(json))
          addImage(json)
      } catch (e) {
        console.warn("Received invalid image:", e);
      }
    },
    'popup_show': (d) => {
      setPopup({ text: d, show: true });
    },
    'popup_hide': () => {
      setPopup(p => ({ ...p, show: false }));
    },
  });

  return (
    <div className={styles.container}>
      {images.map(([img, key]) => {
        return <Fragment key={key}>
          <img key={'image' + key} src={img.url} className={styles.backgroundImage} />
          <h2 key={'title' + key} className={classList(styles.text, styles.title)} >{img.title}</h2>
          <h2 key={'subtitle' + key} className={classList(styles.text, styles.subtitle)} >{img.subtitle}</h2>
        </Fragment>;
      })}
      <Popup content={popup.text} show={popup.show} />
    </div>
  );
}

/**
 * Popup-component that animated based on the `show`-value.
 * Keep in tree at all times set `show` to `false` to hide.
 * 
 * @param params The `content` and whether to `show` the component
 * @returns The component
 */
const Popup = ({ content, show }: { content: string, show: boolean }) => {
  return <div className={classList(styles.popup, show ? styles.popupShow : styles.popupRemove)}>
    <div className={classList(styles.text, styles.popupText, show ? styles.popupTextShow : styles.popupTextRemove)}>{content}</div>
  </div>;
};

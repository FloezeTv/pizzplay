import React, { Fragment, useEffect, useState } from 'react'
import ReactDOM from 'react-dom/client'
import styles from './display.module.css'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Display />
  </React.StrictMode>,
)

const classList = (...classes: string[]): string => classes.join(' ');

type Image = {
  url: string;
  title: string;
  subtitle: string;
};

// For customer-facing displays
export default function Display() {
  const [images, setImages] = useState<[Image, number][]>([]);

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

  // Display some debug images for now
  useEffect(() => {
    let i = 0;
    const intervalId = setInterval(() => {
      addImage({ url: `https://picsum.photos/1920/${1080 + i % 100}`, title: `Item ${i}`, subtitle: `Only $${(i * 431 % 4079) / 100}` });
      i += 1;
    }, 5000);
    return () => clearInterval(intervalId);
  }, []);
  // =================================

  return (
    <div className={styles.container}>
      {images.map(([img, key]) => {
        return <Fragment key={key}>
          <img key={'image' + key} src={img.url} className={styles.backgroundImage} />
          <h2 key={'title' + key} className={classList(styles.text, styles.title)} >{img.title}</h2>
          <h2 key={'subtitle' + key} className={classList(styles.text, styles.subtitle)} >{img.subtitle}</h2>
        </Fragment>;
      })}
    </div>
  );
}

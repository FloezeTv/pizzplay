import { useEffect } from "react";

export const eventListener = (source: string, listeners: { [key: string]: (data: string) => unknown }) => {
	const events = new EventSource(source);
	Object.entries(listeners).forEach(([key, listener]) => events.addEventListener(key, (e) => listener(e.data)));
}

export const useEventListener = (source: string, listeners: { [key: string]: (data: string) => unknown }) => {
	useEffect(() => {
		const events = new EventSource(source);
		Object.entries(listeners).forEach(([key, listener]) => events.addEventListener(key, (e) => listener(e.data)));
		return () => events.close();
	});
}
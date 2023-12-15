import React, { useEffect, useState } from 'react'
import ReactDOM from 'react-dom/client'
import styles from './admin.module.css'
import './index.css'
import { classList } from './utils'
import { useHotkeys } from 'react-hotkeys-hook';

ReactDOM.createRoot(document.getElementById('root')!).render(
	<React.StrictMode>
		<Admin />
	</React.StrictMode>,
)

type OrderType = 'pizza' | 'flammkuchen';

type Order = {
	type: OrderType,
	number: number,
};

export default function Admin() {
	// Functionality:
	// - P: add pizza
	// - F: add flammkuchen
	// - Shift + P: serve pizza
	// - Shift + F: serve flammkuchen
	// - S: Skip number
	// - Shift + S: Unskip number

	type State = {
		currentNumber: number,
		outgoingNumbers: number[],
		incomingNumbers: number[],
		waiting: Order[],
	};

	const [state, setState] = useState<State>({
		currentNumber: parseInt(localStorage.getItem('numberingStart') ?? '0'),
		outgoingNumbers: [],
		incomingNumbers: [],
		waiting: [],
	});
	const [showOutgoingNumbers, setShowOutgoingNumbers] = useState<boolean>(false);
	const [showIncomingNumbers, setShowIncomingNumbers] = useState<boolean>(false);

	// Hide selected numbers 5 seconds after start
	useEffect(() => {
		const timeoutIncoming = setTimeout(() => setShowIncomingNumbers(false), 5000);
		const timeoutOutgoing = setTimeout(() => setShowOutgoingNumbers(false), 5000);
		return () => {
			clearTimeout(timeoutIncoming);
			clearTimeout(timeoutOutgoing);
		};
	}, [showIncomingNumbers, showOutgoingNumbers]);

	const tellServerAboutOrder = (_type: OrderType, number: number) => fetch(`/orders/${number}`, { method: 'POST' });
	const order = (type: OrderType) => {
		setState(s => {
			tellServerAboutOrder(type, s.currentNumber);
			localStorage.setItem('numberingStart', (s.currentNumber + 1).toString());
			return ({
				...s,
				currentNumber: s.currentNumber + 1,
				outgoingNumbers: [...(showOutgoingNumbers ? s.outgoingNumbers : []), s.currentNumber],
				waiting: [...s.waiting, { type: type, number: s.currentNumber }],
			});
		});
		setShowOutgoingNumbers(true);
	}

	const tellServerAboutServing = (number: number) => fetch(`/orders/${number}`, { method: 'DELETE' });

	const serve = (type: OrderType) => {
		setState(s => {
			const nextOrderIdx = s.waiting.findIndex((order) => order.type == type);
			if (nextOrderIdx < 0) {
				console.warn('Tried to serve unordered meals');
				return s;
			}
			const nextOrder = s.waiting[nextOrderIdx];
			const waiting = [...s.waiting];
			waiting.splice(nextOrderIdx, 1);
			tellServerAboutServing(nextOrder.number);
			return { ...s, waiting: waiting, incomingNumbers: [...(showIncomingNumbers ? s.incomingNumbers : []), nextOrder.number] };
		});
		setShowIncomingNumbers(true);
	}

	const skip = (number: number) => {
		if (number < 0) {
			console.warn("Negative skipping currently displays buggy results...");
		}
		setState(s => {
			return ({
				...s,
				currentNumber: s.currentNumber + number,
				outgoingNumbers: [...(showOutgoingNumbers ? s.outgoingNumbers : []), s.currentNumber]
			});
		});
		setShowOutgoingNumbers(true);
	}

	// Order pizza
	useHotkeys('p', () => order('pizza'));
	useHotkeys('f', () => order('flammkuchen'));
	useHotkeys('shift+p', () => serve('pizza'));
	useHotkeys('shift+f', () => serve('flammkuchen'));
	useHotkeys('s', () => skip(1));
	useHotkeys('shift+s', () => skip(-1));

	return <div className={styles.container + ' ' + styles.dark}>
		<div className={styles.handoutContainer}>
			<HandoutDisplay numbers={[...state.outgoingNumbers]} show={showOutgoingNumbers} />
			<HandoutDisplay className={styles.handoutGreen} numbers={[...state.incomingNumbers]} show={showIncomingNumbers} />
		</div>
		<ButtonPanel buttons={[
			['Order Pizza', () => order('pizza')],
			['Order Flammkuchen', () => order('flammkuchen')],
			['Serve Pizza', () => serve('pizza')],
			['Serve Flammkuchen', () => serve('flammkuchen')],
			['Skip', () => skip(1)],
			['Unskip', () => skip(-1)],
		]} />
		<WaitList onClick={console.log} waiting={[...state.waiting]} />
	</div>;
}

const HandoutDisplay = ({ className, numbers, show }: { className?: string, numbers: number[], show: boolean }) => {
	/**
	 * Converts an array of numbers to a nice range-representation.
	 * Will also sort the numbers.
	 * @param numbers numbers to convert
	 * @returns a string containing a nice range-representation of the passed numbers
	 */
	const toString = (numbers: number[]): string => {
		numbers.sort((a, b) => a - b);
		const ranges: [number, number][] = [];
		for (const n of numbers) {
			if (ranges.length == 0) {
				ranges.push([n, 1]);
			} else {
				const [last_start, last_num] = ranges[ranges.length - 1];
				if (last_start + last_num === n) {
					ranges[ranges.length - 1][1] += 1;
				} else {
					ranges.push([n, 1]);
				}
			}
		}
		return ranges.map((r) => {
			const [range_start, range_length] = r;
			if (range_length === 1) {
				return range_start.toString();
			} else if (range_length === 2) {
				return `${range_start}, ${range_start + 1}`;
			} else {
				return `${range_start} - ${range_start + range_length - 1}`;
			}
		}).join(', ');
	}

	return <div className={classList(styles.handoutDisplay, show ? styles.handoutDisplayShow : styles.handoutDisplayRemove, className ?? '')}>
		{toString(numbers)}
	</div>;
}

const ButtonPanel = ({ buttons }: { buttons: [string, () => unknown][] }) => {
	return <div className={styles.buttonPanel}>
		{buttons.map(([text, target]) => <button key={text} className={styles.buttonPanelButton} onClick={() => target()}>{text}</button>)}
	</div>
}

const WaitList = ({ className, waiting, onClick }: { className?: string, waiting: Order[], onClick: (index: Order) => unknown }) => {
	return <div className={classList(styles.waitList, className ?? '')}>
		{waiting.map(w => <button key={w.number} className={styles.waitListElement} onClick={() => onClick(w)}>{w.number}</button>)}
	</div>
}
import {
	red,
	green,
	blue,
	black,
	strikethrough,
} from 'https://deno.land/std@0.113.0/fmt/colors.ts';

const h_ansi = {
	red,
	green,
	blue,
	black,
};

const h_unicode = {
	triangle: '▲',
	square: '▆', // █',
	circle: '●',
	star: '✶', // '★'
};

type Color = 'red' | 'green' | 'blue' | 'black';

const h_colors: Record<Color, number> = {
	red: 0.3,
	green: 0.3,
	blue: 0.3,
	black: 0.1,
};

type Shape = 'triangle' | 'square' | 'circle' | 'star';

const h_shapes: Record<Shape, number> = {
	triangle: 0.3,
	square: 0.3,
	circle: 0.3,
	star: 0.1,
};

class Chip {
	protected _si_color: Color;
	protected _si_shape: Shape;

	protected _a_hints_given: Hint[] = [];

	constructor(si_color: Color, si_shape: Shape) {
		this._si_color = si_color;
		this._si_shape = si_shape;
	}

	get color(): Color {
		return this._si_color;
	}

	get shape(): Shape {
		return this._si_shape;
	}

	hint(k_chip_known: Chip) {
		const a_hints = [
			...Object.keys(h_colors)
				.filter(si => si !== k_chip_known.color && si !== this.color)
				.map(si => new ColorHint(si as Color)),
			...Object.keys(h_shapes)
				.filter(si => si !== k_chip_known.shape && si !== this.shape)
				.map(si => new ShapeHint(si as Shape)),
		].filter((g_hint) => {
			const {
				type: si_type,
				value: si_value,
			} = g_hint;

			for(const g_hint_given of this._a_hints_given) {
				// hint already given
				if(si_type === g_hint_given.type && si_value === g_hint_given.value) {
					return false;
				}
			}

			// hint not yet given
			return true;
		});

		const nl_hints = a_hints.length;

		// select hint randomly
		return a_hints[Math.floor(Math.random() * nl_hints)];
	}

	toString(): string {
		// ${ this._si_color } ${ this._si_shape }
		return `${h_ansi[this._si_color](h_unicode[this._si_shape])}`;
	}
}

function depict(si: Color | Shape) {
	if(si in h_colors) {
		return h_ansi[si as Color](si);
	}
	else {
		return h_unicode[si as Shape];
	}
}

class Hint {
	protected _si_type: 'color' | 'shape';
	protected _si_value: Color | Shape;

	constructor(si_type: 'color' | 'shape', si_value: Color | Shape) {
		this._si_type = si_type;
		this._si_value = si_value;
	}

	get type(): 'color' | 'shape' {
		return this._si_type;
	}

	get value(): Color | Shape {
		return this._si_value;
	}

	toString(): string {
		return `hint: bag is NOT ${this._si_value}`;
	}
}

class ColorHint extends Hint {
	constructor(si_color: Color) {
		super('color', si_color);
	}
}

class ShapeHint extends Hint {
	constructor(si_shape: Shape) {
		super('shape', si_shape);
	}
}


class Hand {
	_k_chip: Chip;
	_k_hint: Hint;

	constructor(k_chip: Chip, k_hint: Hint) {
		this._k_chip = k_chip;
		this._k_hint = k_hint;
	}

	get chip(): Chip {
		return this._k_chip;
	}

	get hint(): Hint {
		return this._k_hint;
	}

	toString(): string {
		return `${this._k_chip} [${this._k_hint}]`;
	}
}

class Round {
	_k_player_a: Hand;
	_k_player_b: Hand;
	_k_bag: Chip;

	constructor(k_player_a: Hand, k_player_b: Hand, k_bag: Chip) {
		this._k_player_a = k_player_a;
		this._k_player_b = k_player_b;
		this._k_bag = k_bag;
	}

	toString(): string {
		return `Bag: ${this._k_bag}`
			+ `\n   Player A: ${this._k_player_a}`
			+ `\n   Player B: ${this._k_player_b}`;
	}


	simulate() {
		const k_player_a = this._k_player_a;
		const k_player_b = this._k_player_b;

		const a_colors = Object.keys(h_colors);
		const a_shapes = Object.keys(h_shapes);

		let b_a_truth = false;

		for(const si_color of a_colors) {
			b_a_truth = si_color === k_player_a.chip.color;

		}
	}
}

type Entry = [string, number];

function normalize(a_entries: Entry[]): Entry[] {
	const c_sum = a_entries.reduce((c, [, x]) => c + x, 0);
	return a_entries.map(([si_key, x_value]) => [si_key, x_value / c_sum]);
}

function  choose_random(h_map: Record<string, number>, as_keys: Set<string>) {
	const x_sel = Math.random();
	const a_entries = normalize(Object.entries(h_map).filter(([si_key, x_value]) => as_keys.has(si_key)));

	let c_cumulative = 0;
	for(const [si_key, x_value] of a_entries) {
		c_cumulative += x_value;

		if(c_cumulative <= x_sel) {
			return si_key;
		}
	}

	return a_entries[a_entries.length-1][0];
}

class Repository {
	_as_colors = new Set(Object.keys(h_colors)) as Set<Color>;
	_as_shapes = new Set(Object.keys(h_shapes)) as Set<Shape>;

	chip(): Chip {
		const as_colors = this._as_colors;
		const si_color = choose_random(h_colors, this._as_colors) as Color;
		as_colors.delete(si_color);

		const as_shapes = this._as_shapes;
		const si_shape = choose_random(h_shapes, this._as_shapes) as Shape;
		as_shapes.delete(si_shape);

		return new Chip(si_color, si_shape);
	}

}

(() => {
	const k_repo = new Repository();

	const k_chip_bag = k_repo.chip();
	
	const k_chip_a = k_repo.chip();
	const k_hint_a = k_chip_bag.hint(k_chip_a);
	const k_player_a = new Hand(k_chip_a, k_hint_a);

	const k_chip_b = k_repo.chip();
	const k_hint_b = k_chip_bag.hint(k_chip_b);
	const k_player_b = new Hand(k_chip_b, k_hint_b);

	const k_round = new Round(k_player_a, k_player_b, k_chip_bag);

	console.log(k_round.toString());
})();


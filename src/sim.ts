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

	equals(k_other: Hint): boolean {
		return k_other.type === this._si_type && k_other.value === this._si_value;
	}


	get probability(): number {
		if('color' === this._si_type) {
			return h_colors[this._si_value as Color];
		}
		else if('shape' === this._si_type) {
			return h_shapes[this._si_value as Shape];
		}
		else {
			 return 0;
		}
	}

	applies(k_chip: Chip): boolean {
		const si_type = this._si_type;

		if('color' === si_type) {
			return this._si_value === k_chip.color;
		}
		else if('shape' === si_type) {
			return this._si_value === k_chip.shape;
		}
		else {
			throw new Error('branch should have been unreachable');
		}
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
		const k_bag = this._k_bag;

		const a_expressions = [
			...Object.keys(h_colors).map(si => new ColorHint(si as Color)),
			...Object.keys(h_shapes).map(si => new ShapeHint(si as Shape)),
		];


		const nl_expressions = a_expressions.length;

		// each possible expression from player a
		for(let i_hint_a=0; i_hint_a<nl_expressions; i_hint_a++) {
			const k_hint_a = a_expressions[i_hint_a];

			// whether player a has truthfully shared their knowledge
			let b_a_truth = k_hint_a.applies(k_player_a.chip) || k_hint_a.equals(k_player_a.hint);

			// whether player a's statement is accurate (falsehoods can still be accurate)
			let b_a_accurate = k_hint_a.applies(k_bag);

			// each possible expression from player b
			for(let i_hint_b=0; i_hint_b<nl_expressions; i_hint_b++) {
				const k_hint_b = a_expressions[i_hint_b];

				// whether player b has truthfully shared their knowledge
				let b_b_truth = k_hint_b.applies(k_player_b.chip) || k_hint_b.equals(k_player_b.hint);

				// whether player b's statement is accurate (falsehoods can still be accurate)
				let b_b_accurate = k_hint_b.applies(k_bag);


				// assuming player a trusts the information it received from player b,
				// then process of elimination leaves player a with the following probability
				// of correctly guessing the bag
				{
					const a_guess_a = a_expressions
						.filter(k_hint => !k_hint.applies(k_player_a.chip)
							&& !k_hint.equals(k_hint_a)
							&& !k_hint.equals(k_hint_b));

					const a_guess_a_colors = normalize(a_guess_a
						.filter(k => 'color' === k.type)
						.map(k => [k.value, k.probability]));

					const a_guess_a_shapes = normalize(a_guess_a
						.filter(k => 'shape' === k.type)
						.map(k => [k.value, k.probability]));

					debugger;

					const nl_color_guesses = a_guess_a.filter(k => 'color' === k.type).length;
					const xr_bag = 1 / (nl_color_guesses * (a_guess_a.length - nl_color_guesses));

					console.log(`player a expresses the bag is NOT ${k_hint_a}: ${xr_bag}`);
					debugger;
				}



				// probability that player a guesses opponent correctly

				// probability that player b guesses bag correctly

				// probability that player a guesses opponent correctly

			}
		}
	}
}

type Entry = [string, number];
type CombinedEntry = [[string, string], number];
type AnyEntry = Entry | CombinedEntry;

function normalize(a_entries: Entry[]): Entry[] {
	const c_sum = a_entries.reduce((c, [, x]) => c + x, 0);
	return a_entries.map(([si_key, x_value]) => [si_key, x_value / c_sum]);
}

function choose_random(a_entries: AnyEntry[]): number {
	const x_sel = Math.random();

	let c_cumulative = 0;
	for(let i_entry=0, nl_entries=a_entries.length; i_entry<nl_entries; i_entry++) {
		const [, x_value] = a_entries[i_entry];

		c_cumulative += x_value;

		if(x_sel <= c_cumulative) {
			return i_entry;
		}
	}

	return a_entries.length-1;
}

function select_1(a_things: AnyEntry[]): string | [string, string] {
	const i_thing = choose_random(a_things);

	const [wi_thing, x_prob] = a_things[i_thing];

	// set probabiliity to zero
	a_things[i_thing][1] = 0;

	// redistribute prob to adjacent colors
	a_things[(i_thing + 3) % 4][1] += x_prob * (2 / 5);
	a_things[(i_thing + 1) % 4][1] += x_prob * (3 / 5);

	return wi_thing;
}

function recombine(a_things: Entry[]): CombinedEntry[] {
	const a_combo: CombinedEntry[] = [];

	const nl_things = a_things.length;
	for(let i_a=0; i_a<nl_things; i_a++) {
		const a_thing_a = a_things[i_a];
		for(let i_b=i_a+1; i_b<nl_things; i_b++) {
			const a_thing_b = a_things[i_b];
			a_combo.push([
				[a_thing_a[0], a_thing_b[0]],
				(a_thing_a[1] + a_thing_b[1]) / 2,
			]);
		}
	}

	return a_combo;
}

class Repository {
	_a_colors = Object.entries(h_colors);
	_a_shapes = Object.entries(h_shapes);

	chip(): Chip {
		const si_color = select_1(this._a_colors) as Color;

		const si_shape = select_1(this._a_shapes) as Shape;

		return new Chip(si_color, si_shape);
	}

	pair(): [Chip, Chip] {
		const a_combos_colors = recombine(this._a_colors.filter(([, x]) => x));
		const a_combos_shapes = recombine(this._a_shapes.filter(([, x]) => x));

		const i_combo_color = choose_random(a_combos_colors);
		const [si_color_0, si_color_1] = a_combos_colors[i_combo_color][0];

		const i_combo_shape = choose_random(a_combos_shapes);
		const [si_shape_0, si_shape_1] = a_combos_shapes[i_combo_shape][0];

		const b_sel_color = Math.random() < 0.5;
		let si_color_a = (b_sel_color? si_color_0: si_color_1) as Color;
		let si_color_b = (b_sel_color? si_color_1: si_color_0) as Color;
		
		const b_sel_shape = Math.random() < 0.5;
		let si_shape_a = (b_sel_shape? si_shape_0: si_shape_1) as Shape;
		let si_shape_b = (b_sel_shape? si_shape_1: si_shape_0) as Shape;

		this._a_colors = this._a_colors.filter(([si, x]) => si !== si_color_0 && si !== si_color_1);
		this._a_shapes = this._a_shapes.filter(([si, x]) => si !== si_shape_0 && si !== si_shape_1);

		return [
			new Chip(si_color_a, si_shape_a),
			new Chip(si_color_b, si_shape_b),
		];
	}
}

(() => {

	select_1(Object.entries(h_colors));

	const k_repo = new Repository();

	const k_chip_bag = k_repo.chip();
	
	const [k_chip_a, k_chip_b] = k_repo.pair();

	const k_hint_a = k_chip_bag.hint(k_chip_a);
	const k_player_a = new Hand(k_chip_a, k_hint_a);

	const k_hint_b = k_chip_bag.hint(k_chip_b);
	const k_player_b = new Hand(k_chip_b, k_hint_b);

	const k_round = new Round(k_player_a, k_player_b, k_chip_bag);

	console.log(k_round.toString());

	k_round.simulate();
})();


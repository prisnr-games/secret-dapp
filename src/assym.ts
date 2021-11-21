import {
	red,
	green,
	blue,
	black,
	bold,
	strikethrough,
} from 'https://deno.land/std@0.113.0/fmt/colors.ts';

type DescEntry = [Color, Desc];
type NumEntry = [string, number];

type Color = 'red' | 'green' | 'blue' | 'black';

interface Desc {
	code: string;
	prob: number;
}

const h_colors: Record<Color, Desc> = {
	red: {
		code: 'R',
		prob: 0.3,
	},
	green: {
		code: 'G',
		prob: 0.3,
	},
	blue: {
		code: 'B',
		prob: 0.3,
	},
	black: {
		code: 'K',
		prob: 0.1,
	},
};

const a_tuples = Object.entries(h_colors) as [Color, Desc][];

function redis(a_tuples: DescEntry[], i_elim: number): DescEntry[] {
	const a_redis = a_tuples.map(([si_color, g_desc]) => [si_color, {
		...g_desc,
	}]) as DescEntry[];
	const x_prob = a_redis[i_elim][1].prob;
	a_redis[(i_elim + 3) % 4][1].prob += x_prob * (2 / 5);
	a_redis[(i_elim + 1) % 4][1].prob += x_prob * (3 / 5);
	a_redis.splice(i_elim, 1);
	return a_redis;
}

interface Pair {
	colors: [Color, Color],
	score: number,
};

function pairs(a_redis: DescEntry[]): Pair[] {
	const a_pairs: Pair[] = [];

	const nl_entries = a_redis.length;
	for(let i_a=0; i_a<nl_entries; i_a++) {
		const g_a = a_redis[i_a][1];
		for(let i_b=i_a+1; i_b<nl_entries; i_b++) {
			const g_b = a_redis[i_b][1];
			a_pairs.push({
				colors: [a_redis[i_a][0], a_redis[i_b][0]],
				score: (g_a.prob + g_b.prob) / 2,
			});
		}
	}

	return a_pairs;
}

interface Spread {
	redis: DescEntry[];
	pairs: Pair[];
}

const h_spreads = Object.keys(h_colors).reduce((h_out, si_color, i_color) => {
	const a_redis = redis(a_tuples, i_color);
	return {
		...h_out,
		[si_color]: {
			redis: a_redis,
			pairs: pairs(a_redis),
		},
	};
}, {}) as Record<Color, Spread>;


function normalize(a_entries: NumEntry[]): NumEntry[] {
	const c_sum = a_entries.reduce((c, [, x]) => c + x, 0);
	return a_entries.map(([si_key, x_value]) => [si_key, x_value / c_sum]);
}

const pct = (x_p: number): string => `${Math.round(x_p * 1e4) / 1e2}%`;

{
	const nl_column = 'P(A.X) = nn.nn% '.length;

	const a_tuples = Object.entries(h_colors) as [Color, Desc][];

	// each hint
	for(const si_hint of ['N/A', ...Object.keys(h_colors)]) {
		// visit each color as user's chip
		for(const [si_color_me, g_color_me] of a_tuples) {
			// hint cannot be same as user' chip color
			if(si_hint === si_color_me) continue;

			const h_arbs = {} as Record<Color, number>;
			const h_opps = Object.keys(h_colors).reduce((h_out, si_color) => ({
				...h_out,
				[si_color]: 0,
			}), {}) as Record<Color, number>;

			// visit each color as arb's bag
			for(const [si_color_arb, g_color_arb] of a_tuples) {
				// ref spread
				const g_spread = h_spreads[si_color_arb];

				// score for P(A|I)
				let x_score = 0;

				// arb's bag color is different than user's chip
				if(si_color_me !== si_color_arb && si_hint !== si_color_arb) {
					// each pair in spread
					for(const g_pair of g_spread.pairs) {
						const a_pair = g_pair.colors;

						// pair includes user's color
						if(a_pair.includes(si_color_me)) {
							// produce weigted score
							const x_weighted = g_pair.score * g_color_arb.prob;

							// add weighted score
							x_score += x_weighted;

							// deduce opponent's chip color in this scenario
							const si_color_opp = a_pair.filter(s => s !== si_color_me)[0];

							// opponent's chip is different than hint
							if(si_hint !== si_color_opp) {
								// add to opp score
								h_opps[si_color_opp] += x_weighted;
							}
						}
					}
				}

				// save score
				h_arbs[si_color_arb] = x_score;
			}

			// highest probability score
			let x_max = 0;
			let i_max = -1;
			let i_which = 0;

			// normalize arb score
			let a_line_arb = [];
			console.log(`Given that my chip is ${si_color_me} and my hint is ${si_hint}...`);
			for(const [si_color_norm, x_norm] of normalize(Object.entries(h_arbs)) as [Color, number][]) {
				const s_given = ''; // `|I.${h_colors[si_color_me].code}`;
				a_line_arb.push(`P(A.${h_colors[si_color_norm].code}${s_given}) = ${pct(x_norm)}`.padEnd(nl_column));
				
				if(x_norm > x_max) {
					x_max = x_norm;
					i_max = i_which;
				}

				i_which++;
			}

			// normalize opps score
			let a_line_opp = [];
			for(const [si_color_norm, x_norm] of normalize(Object.entries(h_opps)) as [Color, number][]) {
				const s_given = ''; // `|I.${h_colors[si_color_me].code}`;
				a_line_opp.push(`P(O.${h_colors[si_color_norm].code}${s_given}) = ${pct(x_norm)}`.padEnd(nl_column));
				
				if(x_norm > x_max) {
					x_max = x_norm;
					i_max = i_which;
				}

				i_which++;
			}

			// output to console
			{
				const a_lens = a_line_arb.map(s => s.length);

				const s_inner_arb = a_line_arb.map((s, i) => i === i_max? bold(s): s).join('│ ');
				console.log('\t┌' + a_lens.map(nl => '─'.repeat(nl + 1)).join('┬') + '┐');
				console.log('\t│ ' + s_inner_arb+'│');
				console.log('\t├' + a_lens.map(nl => '─'.repeat(nl + 1)).join('┼') +'┤');

				const nl_arbs = a_line_arb.length;
				const s_inner_opp = a_line_opp.map((s, i) => (i+nl_arbs) === i_max ? bold(s) : s).join('│ ');
				console.log('\t│ ' + s_inner_opp + '│');
				console.log('\t└' + a_lens.map(nl => '─'.repeat(nl + 1)).join('┴') + '┘');
				console.log('');
			}
		}
	}
}


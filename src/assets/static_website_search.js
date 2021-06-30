
class SpectralBloomFilter {
	constructor(search_item) {
		this.sbf_base2p15 = search_item["sbf_base2p15"];
		this.n_hash_functions = search_item["n_hash_functions"];
		this.width = search_item["width"];
		this.url = search_item["url"];
		this.size = search_item["size"];
		this.title = search_item["title"];
	}
	hash_fn(key, seed) {
		var remainder, bytes, h1, h1b, c1, c1b, c2, c2b, k1, i;

		remainder = key.length & 3; // key.length % 4
		bytes = key.length - remainder;
		h1 = seed;
		c1 = 0xcc9e2d51;
		c2 = 0x1b873593;
		i = 0;

		while (i < bytes) {
			k1 =
				((key.charCodeAt(i) & 0xff)) |
				((key.charCodeAt(++i) & 0xff) << 8) |
				((key.charCodeAt(++i) & 0xff) << 16) |
				((key.charCodeAt(++i) & 0xff) << 24);
			++i;

			k1 = ((((k1 & 0xffff) * c1) + ((((k1 >>> 16) * c1) & 0xffff) << 16))) & 0xffffffff;
			k1 = (k1 << 15) | (k1 >>> 17);
			k1 = ((((k1 & 0xffff) * c2) + ((((k1 >>> 16) * c2) & 0xffff) << 16))) & 0xffffffff;

			h1 ^= k1;
			h1 = (h1 << 13) | (h1 >>> 19);
			h1b = ((((h1 & 0xffff) * 5) + ((((h1 >>> 16) * 5) & 0xffff) << 16))) & 0xffffffff;
			h1 = (((h1b & 0xffff) + 0x6b64) + ((((h1b >>> 16) + 0xe654) & 0xffff) << 16));
		}

		k1 = 0;

		switch (remainder) {
			case 3:
				k1 ^= (key.charCodeAt(i + 2) & 0xff) << 16;
			case 2:
				k1 ^= (key.charCodeAt(i + 1) & 0xff) << 8;
			case 1:
				k1 ^= (key.charCodeAt(i) & 0xff);

				k1 = (((k1 & 0xffff) * c1) + ((((k1 >>> 16) * c1) & 0xffff) << 16)) & 0xffffffff;
				k1 = (k1 << 15) | (k1 >>> 17);
				k1 = (((k1 & 0xffff) * c2) + ((((k1 >>> 16) * c2) & 0xffff) << 16)) & 0xffffffff;
				h1 ^= k1;
		}

		h1 ^= key.length;

		h1 ^= h1 >>> 16;
		h1 = (((h1 & 0xffff) * 0x85ebca6b) + ((((h1 >>> 16) * 0x85ebca6b) & 0xffff) << 16)) & 0xffffffff;
		h1 ^= h1 >>> 13;
		h1 = ((((h1 & 0xffff) * 0xc2b2ae35) + ((((h1 >>> 16) * 0xc2b2ae35) & 0xffff) << 16))) & 0xffffffff;
		h1 ^= h1 >>> 16;

		return h1 >>> 0;
	}
	base2p15_decode(base2p15) {
		let bit_string = "";
		let offset = 0xa1;
		let padding = parseInt(base2p15[0], 16);
		for (let character of base2p15.slice(1, -1)) {
			character = character.charCodeAt() - offset;
			let bits = character.toString(2).padStart(15, "0");
			bit_string += bits;
		}

		let character = base2p15.slice(-1).charCodeAt() - offset;
		let bits = character.toString(2).padStart(15, "0").slice(0, 15 - padding);
		bit_string += bits;
		return bit_string;
	}
	base2p15_get_range(base2p15,start, end) {
		let range_str = base2p15.slice(1).slice(Math.floor(start / 15), Math.floor(end / 15) + 1);
		let end_pad = (15 - end % 15).toString(16);
		let start_pad = start % 15;
		let decoded = this.base2p15_decode(end_pad + range_str).slice(start_pad);
		return decoded;
	}
	get_counter(i) {
		let l = i * this.width;
		let r = (i + 1) * this.width;
		return parseInt(this.base2p15_get_range(this.sbf_base2p15,l, r), 2);
	}
	get_frequency(word) {
		let mn = Infinity;
		for (let i = 0; i < this.n_hash_functions; ++i) {
			let idx = this.hash_fn(word, i) % this.size;
			mn = Math.min(this.get_counter(idx), mn);
		}
		return mn;
	}
}
var SEARCH_INDEX = UNIQUE_SEARCH_INDEX_PLACEHOLDER.map(e => new SpectralBloomFilter(e))

function search_sbf(words) {
	let results = []
	for (let item of SEARCH_INDEX) {
		let score = words.map(w => item.get_frequency(w)).reduce((x, y) => x + y);
		if (score > 0)
			results.push({
				"title": item.title,
				"url": item.url,
				"score": score
			});
	}
	results.sort((a, b) => a.score < b.score ? 1 : -1);
	return results;
}

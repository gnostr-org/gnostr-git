#define EMITTED_DIFF_SYMBOL_INIT {NULL}
#define EMITTED_DIFF_SYMBOLS_INIT {NULL, 0, 0}
	struct hashmap_entry ent;
static void moved_block_clear(struct moved_block *b)
{
	memset(b, 0, sizeof(*b));
}

			    const struct emitted_diff_symbol *b,
			    int *out)
{
	int a_len = a->len,
	    b_len = b->len,
	    a_off = a->indent_off,
	    a_width = a->indent_width,
	    b_off = b->indent_off,
	int delta;

	if (a_width == INDENT_BLANKLINE && b_width == INDENT_BLANKLINE) {
		*out = INDENT_BLANKLINE;
		return 1;
	}
	if (a->s == DIFF_SYMBOL_PLUS)
		delta = a_width - b_width;
	else
		delta = b_width - a_width;
	if (a_len - a_off != b_len - b_off ||
	    memcmp(a->line + a_off, b->line + b_off, a_len - a_off))
		return 0;

	*out = delta;

	return 1;
static int cmp_in_block_with_wsd(const struct diff_options *o,
				 const struct moved_entry *cur,
				 const struct moved_entry *match,
				 struct moved_block *pmb,
				 int n)
{
	struct emitted_diff_symbol *l = &o->emitted_symbols->buf[n];
	int al = cur->es->len, bl = match->es->len, cl = l->len;
	const char *a = cur->es->line,
		   *b = match->es->line,
		   *c = l->line;
	int a_off = cur->es->indent_off,
	    a_width = cur->es->indent_width,
	    c_off = l->indent_off,
	    c_width = l->indent_width;
	/*
	 * We need to check if 'cur' is equal to 'match'.  As those
	 * are from the same (+/-) side, we do not need to adjust for
	 * indent changes. However these were found using fuzzy
	 * matching so we do have to check if they are equal. Here we
	 * just check the lengths. We delay calling memcmp() to check
	 * the contents until later as if the length comparison for a
	 * and c fails we can avoid the call all together.
	 */
	if (al != bl)
	/* If 'l' and 'cur' are both blank then they match. */
	if (a_width == INDENT_BLANKLINE && c_width == INDENT_BLANKLINE)
	 * match those of the current block and that the text of 'l' and 'cur'
	 * after the indentation match.
	if (cur->es->s == DIFF_SYMBOL_PLUS)
		delta = a_width - c_width;
	else
		delta = c_width - a_width;
	return !(delta == pmb->wsd && al - a_off == cl - c_off &&
		 !memcmp(a, b, al) && !
		 memcmp(a + a_off, c + c_off, al - a_off));
static int moved_entry_cmp(const void *hashmap_cmp_fn_data,
			   const struct hashmap_entry *eptr,
			   const struct hashmap_entry *entry_or_key,
			   const void *keydata)
	const struct moved_entry *a, *b;
	a = container_of(eptr, const struct moved_entry, ent);
	b = container_of(entry_or_key, const struct moved_entry, ent);

	if (diffopt->color_moved_ws_handling &
	    COLOR_MOVED_WS_ALLOW_INDENTATION_CHANGE)
		/*
		 * As there is not specific white space config given,
		 * we'd need to check for a new block, so ignore all
		 * white space. The setup of the white space
		 * configuration for the next block is done else where
		 */
		flags |= XDF_IGNORE_WHITESPACE;
	return !xdiff_compare_lines(a->es->line, a->es->len,
				    b->es->line, b->es->len,
				    flags);
static struct moved_entry *prepare_entry(struct diff_options *o,
					 int line_no)
	struct moved_entry *ret = xmalloc(sizeof(*ret));
	struct emitted_diff_symbol *l = &o->emitted_symbols->buf[line_no];
	unsigned int hash = xdiff_hash_string(l->line, l->len, flags);
	hashmap_entry_init(&ret->ent, hash);
	ret->es = l;
	ret->next_line = NULL;
	return ret;
}
static void add_lines_to_move_detection(struct diff_options *o,
					struct hashmap *add_lines,
					struct hashmap *del_lines)
	int n;
		struct hashmap *hm;
		struct moved_entry *key;
		switch (o->emitted_symbols->buf[n].s) {
		case DIFF_SYMBOL_PLUS:
			hm = add_lines;
			break;
		case DIFF_SYMBOL_MINUS:
			hm = del_lines;
			break;
		default:
			fill_es_indent_data(&o->emitted_symbols->buf[n]);
		key = prepare_entry(o, n);
		if (prev_line && prev_line->es->s == o->emitted_symbols->buf[n].s)
			prev_line->next_line = key;
		hashmap_add(hm, &key->ent);
		prev_line = key;
				struct moved_entry *match,
				struct hashmap *hm,
				int pmb_nr)
	int i;
	for (i = 0; i < pmb_nr; i++) {
		if (cur && !hm->cmpfn(o, &cur->ent, &match->ent, NULL)) {
			pmb[i].match = cur;
		} else {
			pmb[i].match = NULL;
		}
	}
}
static void pmb_advance_or_null_multi_match(struct diff_options *o,
					    struct moved_entry *match,
					    struct hashmap *hm,
					    struct moved_block *pmb,
					    int pmb_nr, int n)
{
	int i;
	char *got_match = xcalloc(1, pmb_nr);

	hashmap_for_each_entry_from(hm, match, ent) {
		for (i = 0; i < pmb_nr; i++) {
			struct moved_entry *prev = pmb[i].match;
			struct moved_entry *cur = (prev && prev->next_line) ?
					prev->next_line : NULL;
			if (!cur)
				continue;
			if (!cmp_in_block_with_wsd(o, cur, match, &pmb[i], n))
				got_match[i] |= 1;
		}
	}
	for (i = 0; i < pmb_nr; i++) {
		if (got_match[i]) {
			/* Advance to the next line */
			pmb[i].match = pmb[i].match->next_line;
		} else {
			moved_block_clear(&pmb[i]);
	free(got_match);
}
static int shrink_potential_moved_blocks(struct moved_block *pmb,
					 int pmb_nr)
	int lp, rp;
	/* Shrink the set of potential block to the remaining running */
	for (lp = 0, rp = pmb_nr - 1; lp <= rp;) {
		while (lp < pmb_nr && pmb[lp].match)
			lp++;
		/* lp points at the first NULL now */

		while (rp > -1 && !pmb[rp].match)
			rp--;
		/* rp points at the last non-NULL */

		if (lp < pmb_nr && rp > -1 && lp < rp) {
			pmb[lp] = pmb[rp];
			memset(&pmb[rp], 0, sizeof(pmb[rp]));
			rp--;
			lp++;
		}
	/* Remember the number of running sets */
	return rp + 1;
		o->emitted_symbols->buf[n - i].flags &= ~DIFF_SYMBOL_MOVED_LINE;
				struct hashmap *add_lines,
				struct hashmap *del_lines)
		struct hashmap *hm = NULL;
		struct moved_entry *key;
		enum diff_symbol last_symbol = 0;
			hm = del_lines;
			key = prepare_entry(o, n);
			match = hashmap_get_entry(hm, key, ent, NULL);
			free(key);
			hm = add_lines;
			key = prepare_entry(o, n);
			match = hashmap_get_entry(hm, key, ent, NULL);
			free(key);
		if (!match) {
			int i;

			adjust_last_block(o, n, block_length);
			for(i = 0; i < pmb_nr; i++)
				moved_block_clear(&pmb[i]);
			last_symbol = l->s;
			last_symbol = l->s;
		if (o->color_moved_ws_handling &
		    COLOR_MOVED_WS_ALLOW_INDENTATION_CHANGE)
			pmb_advance_or_null_multi_match(o, match, hm, pmb, pmb_nr, n);
		else
			pmb_advance_or_null(o, match, hm, pmb, pmb_nr);

		pmb_nr = shrink_potential_moved_blocks(pmb, pmb_nr);
			/*
			 * The current line is the start of a new block.
			 * Setup the set of potential blocks.
			 */
			hashmap_for_each_entry_from(hm, match, ent) {
				ALLOC_GROW(pmb, pmb_nr + 1, pmb_alloc);
				if (o->color_moved_ws_handling &
				    COLOR_MOVED_WS_ALLOW_INDENTATION_CHANGE) {
					if (compute_ws_delta(l, match->es,
							     &pmb[pmb_nr].wsd))
						pmb[pmb_nr++].match = match;
				} else {
					pmb[pmb_nr].wsd = 0;
					pmb[pmb_nr++].match = match;
				}
			}
			if (adjust_last_block(o, n, block_length) &&
			    pmb_nr && last_symbol != l->s)
		last_symbol = l->s;
	for(n = 0; n < pmb_nr; n++)
		moved_block_clear(&pmb[n]);
#define DIFF_SYMBOL_MOVED_LINE_ZEBRA_MASK \
  (DIFF_SYMBOL_MOVED_LINE | DIFF_SYMBOL_MOVED_LINE_ALT)
	struct emitted_diff_symbol e = {line, len, flags, 0, 0, s};
		die(_("--name-only, --name-status, --check and -s are mutually exclusive"));
		die(_("-G, -S and --find-object are mutually exclusive"));
		die(_("-G and --pickaxe-regex are mutually exclusive, use --pickaxe-regex with -S"));
		die(_("--pickaxe-all and --find-object are mutually exclusive, use --pickaxe-all with -G and -S"));
static const char diff_status_letters[] = {
	DIFF_STATUS_ADDED,
	DIFF_STATUS_COPIED,
	DIFF_STATUS_DELETED,
	DIFF_STATUS_MODIFIED,
	DIFF_STATUS_RENAMED,
	DIFF_STATUS_TYPE_CHANGED,
	DIFF_STATUS_UNKNOWN,
	DIFF_STATUS_UNMERGED,
	DIFF_STATUS_FILTER_AON,
	DIFF_STATUS_FILTER_BROKEN,
	'\0',
};

static unsigned int filter_bit['Z' + 1];

static void prepare_filter_bits(void)
{
	int i;

	if (!filter_bit[DIFF_STATUS_ADDED]) {
		for (i = 0; diff_status_letters[i]; i++)
			filter_bit[(int) diff_status_letters[i]] = (1 << i);
	}
}

static unsigned filter_bit_tst(char status, const struct diff_options *opt)
{
	return opt->filter & filter_bit[(int) status];
}

unsigned diff_filter_bit(char status)
{
	prepare_filter_bits();
	return filter_bit[(int) status];
}

	/*
	 * If there is a negation e.g. 'd' in the input, and we haven't
	 * initialized the filter field with another --diff-filter, start
	 * from full set of bits, except for AON.
	 */
	if (!opt->filter) {
		for (i = 0; (optch = optarg[i]) != '\0'; i++) {
			if (optch < 'a' || 'z' < optch)
				continue;
			opt->filter = (1 << (ARRAY_SIZE(diff_status_letters) - 1)) - 1;
			opt->filter &= ~filter_bit[DIFF_STATUS_FILTER_AON];
			break;
		}
	}

			opt->filter &= ~bit;
		  N_("Output to a specific file"),
	if (diff_unmodified_pair(p))
		return; /* no tree diffs in patch format */
int diff_queue_is_empty(void)
			struct hashmap add_lines, del_lines;
			if (o->color_moved_ws_handling &
			    COLOR_MOVED_WS_ALLOW_INDENTATION_CHANGE)
				o->color_moved_ws_handling |= XDF_IGNORE_WHITESPACE;

			hashmap_init(&del_lines, moved_entry_cmp, o, 0);
			hashmap_init(&add_lines, moved_entry_cmp, o, 0);

			add_lines_to_move_detection(o, &add_lines, &del_lines);
			mark_color_as_moved(o, &add_lines, &del_lines);
			hashmap_clear_and_free(&add_lines, struct moved_entry,
						ent);
			hashmap_clear_and_free(&del_lines, struct moved_entry,
						ent);
	if (!q->nr)
	const char *argv[3];
	const char **arg = argv;
	*arg++ = pgm;
	*arg++ = temp->name;
	*arg = NULL;
	child.argv = argv;
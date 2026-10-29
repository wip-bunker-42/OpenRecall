use super::library::id;
use crate::{
    App, AppError, AppResult,
    contracts::{library::DeckDetail, spelling::SpellingDeckResult},
    domain::{
        card::CardDetail,
        spelling::{DeckKind, SpellingConfig},
    },
    spelling::{extract::EXTRACTION_POLICY_VERSION, storage},
};
use rusqlite::{OptionalExtension, params};

impl App {
    pub(crate) fn exclude_deleted_spelling_word(&self, card: &CardDetail) -> AppResult<()> {
        let deck: DeckDetail = self.store.get("decks", &card.deck_id)?;
        if deck.kind != DeckKind::Spelling {
            return Ok(());
        }
        let canonical: String = self.store.conn.query_row(
            "SELECT canonical FROM spelling_words WHERE card_id=?1",
            [&card.id],
            |row| row.get(0),
        )?;
        let mut config = self.get_spelling_config(&card.deck_id)?;
        storage::save_exclusion(
            &self.store.conn,
            &crate::domain::spelling::SpellingExclusion {
                deck_id: card.deck_id.clone(),
                canonical,
                excluded_at: self.clock.now().to_rfc3339(),
            },
        )?;
        config.revision = id();
        storage::save_config(&self.store.conn, &config)?;
        Ok(())
    }
    pub(crate) fn list_spelling_exclusions(
        &self,
        deck_id: &str,
        page: u32,
        page_size: u16,
    ) -> AppResult<crate::contracts::cards::Page<crate::domain::spelling::SpellingExclusion>> {
        self.get_spelling_config(deck_id)?;
        if page == 0 || ![10, 20, 50, 100].contains(&page_size) {
            return Err(AppError::field("page", "range"));
        }
        let total = self.store.conn.query_row(
            "SELECT count(*) FROM spelling_exclusions WHERE deck_id=?1",
            [deck_id],
            |row| row.get(0),
        )?;
        let items=self.store.conn.prepare("SELECT canonical,excluded_at FROM spelling_exclusions WHERE deck_id=?1 ORDER BY excluded_at,canonical LIMIT ?2 OFFSET ?3")?
            .query_map(params![deck_id,page_size,(page as i64-1)*page_size as i64],|row|Ok(crate::domain::spelling::SpellingExclusion {deck_id:deck_id.into(),canonical:row.get(0)?,excluded_at:row.get(1)?}))?.collect::<Result<_,_>>()?;
        Ok(crate::contracts::cards::Page {
            items,
            total,
            page,
            page_size,
        })
    }
    pub(crate) fn allow_spelling_word(
        &mut self,
        deck_id: String,
        canonical: String,
        expected_revision: String,
    ) -> AppResult<SpellingDeckResult> {
        let mut config = self.get_spelling_config(&deck_id)?;
        if config.revision != expected_revision {
            return Err(AppError::new("conflict"));
        }
        if crate::spelling::extract::canonical_word(&canonical).as_deref()
            != Some(canonical.as_str())
        {
            return Err(AppError::field("canonical", "invalidSpellingWord"));
        }
        let before = self.store.conn.total_changes();
        if self.store.conn.execute(
            "DELETE FROM spelling_exclusions WHERE deck_id=?1 AND canonical=?2",
            params![deck_id, canonical],
        )? == 0
        {
            return Err(AppError::new("notFound"));
        }
        config.revision = id();
        storage::save_config(&self.store.conn, &config)?;
        let added = if let Some(source) = &config.source_deck_id {
            self.backfill_spelling_source(source)?
        } else {
            0
        };
        self.finish_spelling_collection(&deck_id, added, before)?;
        self.spelling_result(config, added)
    }
    pub(crate) fn sync_spelling_for_content_change(&mut self, card: &CardDetail) -> AppResult<()> {
        let Some(config) = storage::config(&self.store.conn)? else {
            return Ok(());
        };
        let active = config.source_deck_id.as_deref() == Some(card.deck_id.as_str());
        if !active && !storage::observed_card(&self.store.conn, &card.id)? {
            return Ok(());
        }
        let before = self.store.conn.total_changes();
        let added = self.collect_spelling_content(card, active)?;
        self.finish_spelling_collection(&config.deck_id, added, before)
    }
    pub(crate) fn retain_spelling_before_source_delete(
        &mut self,
        card_ids: &[String],
    ) -> AppResult<()> {
        let Some(config) = storage::config(&self.store.conn)? else {
            return Ok(());
        };
        let before = self.store.conn.total_changes();
        for card_id in card_ids {
            if storage::observed_card(&self.store.conn, card_id)? {
                let card: CardDetail = self.store.get("cards", card_id)?;
                self.collect_spelling_content(&card, false)?;
            }
        }
        self.finish_spelling_collection(&config.deck_id, 0, before)
    }
    fn ordinary_spelling_source(&self, source: &str) -> AppResult<DeckDetail> {
        let deck: DeckDetail = self.store.get("decks", source)?;
        if deck.kind != DeckKind::Standard {
            return Err(AppError::new("invalidSpellingSource"));
        }
        Ok(deck)
    }
    pub(crate) fn get_spelling_config(&self, deck: &str) -> AppResult<SpellingConfig> {
        let config =
            storage::config(&self.store.conn)?.ok_or_else(|| AppError::new("notSpellingDeck"))?;
        if config.deck_id != deck {
            return Err(AppError::new("notSpellingDeck"));
        }
        Ok(config)
    }
    pub(crate) fn create_spelling_deck(
        &mut self,
        source_deck_id: String,
    ) -> AppResult<SpellingDeckResult> {
        let existing: Option<String> = self
            .store
            .conn
            .query_row("SELECT id FROM decks WHERE kind='spelling'", [], |r| {
                r.get(0)
            })
            .optional()?;
        if let Some(existing) = existing {
            let mut error = AppError::new("spellingDeckExists");
            error.context = Some(existing);
            return Err(error);
        }
        let source = self.ordinary_spelling_source(&source_deck_id)?;
        let deck =
            self.create_deck_kind("مجموعة التدرب على الكتابة".into(), None, DeckKind::Spelling)?;
        let config = SpellingConfig {
            deck_id: deck.id.clone(),
            source_deck_id: Some(source.id.clone()),
            source_deck_name: Some(source.name),
            extraction_policy_version: EXTRACTION_POLICY_VERSION,
            revision: id(),
        };
        storage::save_config(&self.store.conn, &config)?;
        let before = self.store.conn.total_changes();
        let added = self.backfill_spelling_source(&source.id)?;
        self.finish_spelling_collection(&config.deck_id, added, before)?;
        self.spelling_result(config, added)
    }
    pub(crate) fn set_spelling_source(
        &mut self,
        deck_id: String,
        source_deck_id: String,
        expected_revision: String,
    ) -> AppResult<SpellingDeckResult> {
        let mut config = self.get_spelling_config(&deck_id)?;
        if config.revision != expected_revision {
            return Err(AppError::new("conflict"));
        }
        let source = self.ordinary_spelling_source(&source_deck_id)?;
        let before = self.store.conn.total_changes();
        config.source_deck_id = Some(source.id.clone());
        config.source_deck_name = Some(source.name);
        config.revision = id();
        storage::save_config(&self.store.conn, &config)?;
        let added = self.backfill_spelling_source(&source.id)?;
        self.finish_spelling_collection(&deck_id, added, before)?;
        self.spelling_result(config, added)
    }
    pub(crate) fn rescan_spelling_source(
        &mut self,
        deck_id: String,
        expected_revision: String,
    ) -> AppResult<SpellingDeckResult> {
        let config = self.get_spelling_config(&deck_id)?;
        if config.revision != expected_revision {
            return Err(AppError::new("conflict"));
        }
        let source = config
            .source_deck_id
            .as_deref()
            .ok_or_else(|| AppError::new("noSpellingSource"))?;
        self.ordinary_spelling_source(source)?;
        let before = self.store.conn.total_changes();
        let added = self.backfill_spelling_source(source)?;
        self.finish_spelling_collection(&deck_id, added, before)?;
        self.spelling_result(config, added)
    }
    fn spelling_result(
        &self,
        config: SpellingConfig,
        added_words: u32,
    ) -> AppResult<SpellingDeckResult> {
        Ok(SpellingDeckResult {
            deck: self.deck_detail(&config.deck_id)?,
            config,
            added_words,
        })
    }
    pub(crate) fn finish_spelling_collection(
        &self,
        deck: &str,
        added: u32,
        before: u64,
    ) -> AppResult<()> {
        storage::prune_snapshots(&self.store.conn)?;
        if added > 0 {
            self.touch_deck(deck)?;
        } else if self.store.conn.total_changes() > before {
            self.store.touch()?;
        }
        Ok(())
    }
    fn backfill_spelling_source(&mut self, source: &str) -> AppResult<u32> {
        let mut cursor = (i64::MIN, String::new());
        let mut added = 0;
        loop {
            let batch:Vec<(i64,String,String)>=self.store.conn.prepare("SELECT created_ms,id,payload FROM cards WHERE deck_id=?1 AND (created_ms,id)>(?2,?3) ORDER BY created_ms,id LIMIT 128")?
                .query_map(params![source,cursor.0,&cursor.1],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<Result<_,_>>()?;
            if batch.is_empty() {
                break;
            }
            for (created, card_id, payload) in batch {
                let card: CardDetail = serde_json::from_str(&payload)?;
                added += self.collect_spelling_content(&card, true)?;
                cursor = (created, card_id);
            }
        }
        Ok(added)
    }
}

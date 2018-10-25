macro_rules! action_helper {
    ( @emit_lex_unit_with_raw_inclusive | $self:tt, $input:ident |> $token:expr ) => {
        action_helper!(@emit_lex_unit_with_raw |$self, $input|> $token, input!(@pos $self) + 1 )
    };

    ( @emit_lex_unit_with_raw_exclusive | $self:tt, $input:ident |> $token:expr ) => {
        action_helper!(@emit_lex_unit_with_raw |$self, $input|> $token, input!(@pos $self) )
    };

    ( @emit_lex_unit_with_raw | $self:tt, $input:ident |> $token:expr, $end:expr ) => ({
        let raw_range = Some(Range {
            start: $self.lex_unit_start,
            end: $end,
        });

        $self.lex_unit_start = $end;

        action_helper!(@emit_lex_unit |$self|>
            $token,
            raw_range,
            $input
        )
    });

    ( @emit_lex_unit | $self:tt |> $token:expr, $raw:expr, $input:ident ) => ({
        let lex_unit = LexUnit::new($input, $token, $raw);

        $self.lex_unit_handler.handle(&lex_unit);

        lex_unit
    });

    ( @finish_token_part | $self:tt, $input:ident |> $part:ident ) => {
        $part.start = $self.token_part_start;
        $part.end = input!(@pos $self);
    };

    ( @finish_opt_token_part | $self:tt, $input:ident |> $part:ident ) => {
        *$part = Some({
            let mut $part = Range::default();

            action_helper!(@finish_token_part |$self, $input|> $part);

            $part
        });
    };

    ( @finish_attr_part | $self:tt, $input:ident |> $part:ident ) => {
        if let Some(AttributeView { ref mut $part, .. }) = $self.current_attr {
            action_helper!(@finish_token_part |$self, $input|> $part);
        }
    };

    ( @update_tag_part | $self:tt |> $part:ident, $action:block ) => {
        match $self.current_token {
            Some(TokenView::StartTag { ref mut $part, .. }) |
            Some(TokenView::EndTag { ref mut $part, .. }) => $action
            _ => unreachable!("Current token should always be a start or an end tag at this point")
        }
    };

    ( @switch_state | $self:tt |> $state:expr ) => {
        $self.state = $state;
        $self.state_enter = true;
        return Ok(ParsingLoopDirective::Continue);
    };

    ( @notify_text_parsing_mode_change | $self:tt |> $mode:expr ) => {
        #[cfg(feature = "testing_api")]
        {
            if let Some(ref mut text_parsing_mode_change_handler) =
                $self.text_parsing_mode_change_handler
            {
                text_parsing_mode_change_handler.handle(TextParsingModeSnapshot {
                    mode: $mode,
                    last_start_tag_name_hash: $self.last_start_tag_name_hash,
                });
            }
        }
    };
}

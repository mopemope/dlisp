# エージェント評価

`cases.json` は日本語の routing 8 件と、既知の不具合を注入する repair 2 件。モデル能力全般を証明するベンチマークではなく、AGENTS.md / Skills / ツールの変更による回帰を見つける小さな評価セット。

## 実行

```bash
python3 scripts/run_agent_eval.py --list
python3 scripts/run_agent_eval.py --case cli-routing --ref HEAD --model <利用するモデル> --effort medium --repeat 3 --output-dir /tmp/dlisp-eval-candidate
```

- `--list` はモデルを呼ばない。実行には Codex CLI の認証済み環境が必要で、モデル使用量が発生する。
- `--ref` のコミットから `git archive` で一時 checkout を作る。未コミットの変更は評価に含まれない。比較したい修正をコミットした後、その revision を指定する。
- baseline と candidate で、モデル・effort・ケース・繰り返し回数・環境・グローバル指示/Skills を揃える。fixture が異なる場合はその差も記録する。
- checkout 内の .agents / .claude のリンクは相対パスのまま保持する。元の作業ツリーには不具合を注入しない。
- 一時 checkout には baseline commit を置くため、`git diff HEAD` や検証プランの既定コマンドも利用できる。agent process の標準入力は閉じ、親プロセスの入力を追加 prompt として読み込ませない。
- timeout は `--timeout` 秒。Codex とその子プロセスを終了させる。検証コマンドにも同じ上限を適用する。
- 一時ビルドの既定値は debug 情報なし・incremental なし・2 jobs。agent と grader に同じ設定を渡し、明示された環境変数は優先する。repair の prompt は対象検証だけを完了条件とし、独立した採点は host 側で実施する。全体テストは repo の最終確認で行う。

## 判定と記録

各実行の `request.json`、`trace.jsonl`、`stderr.log`、`changes.patch`、`result.json` を保存し、全体を `summary.json` にまとめる。新規未追跡ファイルは result の changed_files に記録するが patch には含まれない。

- routing: 変更がないこと、成功した read コマンドに期待 Skill のパスがあること、最終回答に根拠パスがあることを判定する。read コマンドの抽出は補助指標であり、理解や説明の正しさは人が確認する。カスタム read ツールは記録形式によって検出できない場合がある。
- repair: 許可範囲内の変更かを確認し、runtime の実テスト、またはホスト側 ABI checker と core のビルド検査で正しさを判定する。runtime テストは実行前のコピーを復元してから採点する。テスト・宣言の削除による見かけの成功を許容しない。
- 途中失敗、完了イベント欠落、不正 JSONL、timeout は不合格。staged / unstaged / untracked の変更を検出する。
- `turn.completed.usage` の入力・キャッシュ入力・出力を別々に集計する。取得できない値は null。キャッシュ入力は入力の内数なので、総トークンに二重加算しない。
- `tokens_per_success` は失敗分も含む入力＋出力を成功件数で割った値。routing と repair の成功数も分けて見る。料金はモデル単価とキャッシュで異なるため、この値を金額とみなさない。

## 拡張

実際の失敗が起きたら、再現可能な mutation と独立した grader を追加する。修正用の評価では、期待値や採点テストを修正対象にしない。まず同じ入力を修正前に失敗、修正後に成功と判定できるかを確認する。

モデル比較は複数回で行い、未使用ケースも別に用意して評価セットへの過適合を避ける。今回の 2 repair ケースでは parser・closure・concurrency の実装能力までは測れないため、それらの実作業で得た回帰ケースを優先して追加する。

方式の参照: [OpenAI の Skill 評価ガイド](https://developers.openai.com/blog/eval-skills)、[Skills の配置と段階的な読み込み](https://learn.chatgpt.com/docs/build-skills)。

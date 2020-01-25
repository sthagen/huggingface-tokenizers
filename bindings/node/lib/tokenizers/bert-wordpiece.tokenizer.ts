import { promisify } from "util";

import { wordPieceDecoder } from "../bindings/decoders";
import { Model, WordPiece } from "../bindings/models";
import { bertNormalizer } from "../bindings/normalizers";
import { bertProcessing } from "../bindings/post-processors";
import { bertPreTokenizer } from "../bindings/pre-tokenizers";
import { Tokenizer } from "../bindings/tokenizer";
import { wordPieceTrainer } from "../bindings/trainers";
import { BaseTokenizer } from "./base.tokenizer";

export interface BertWordPieceOptions {
  /**
   * @default true
   */
  addSpecialTokens?: boolean;
  /**
   * @default true
   */
  cleanText?: boolean;
  /**
   * @default "[CLS]"
   */
  clsToken?: string;
  /**
   * @default true
   */
  handleChineseChars?: boolean;
  /**
   * @default true
   */
  lowercase?: boolean;
  /**
   * @default "[SEP]"
   */
  sepToken?: string;
  /**
   * @default true
   */
  stripAccents?: boolean;
  /**
   * @default "[UNK]"
   */
  unkToken?: string;
  vocabFile?: string;
  /**
   * @default "##"
   */
  wordpiecesPrefix?: string;
}

export interface BertWordPieceTrainOptions {
  /**
   * @default []
   */
  initialAlphabet?: string[];
  /**
   * @default 1000
   */
  limitAlphabet?: number;
  /**
   * @default 2
   */
  minFrequency?: number;
  /**
   * @default true
   */
  showProgress?: boolean;
  /**
   * @default ["[UNK]", "[SEP]", "[CLS]"]
   */
  specialTokens?: string[];
  /**
   * @default 30000
   */
  vocabSize?: number;
  /**
   * @default "##"
   */
  wordpiecesPrefix?: string;
}

/**
 * Bert WordPiece Tokenizer
 */
export class BertWordPieceTokenizer extends BaseTokenizer {
  private static readonly defaultBertOptions: Required<
    Omit<BertWordPieceOptions, "vocabFile">
  > & { vocabFile?: string } = {
    addSpecialTokens: true,
    cleanText: true,
    clsToken: "[CLS]",
    handleChineseChars: true,
    lowercase: true,
    sepToken: "[SEP]",
    stripAccents: true,
    unkToken: "[UNK]",
    wordpiecesPrefix: "##"
  };

  private readonly defaultTrainOptions: Required<BertWordPieceTrainOptions> = {
    initialAlphabet: [],
    limitAlphabet: 1000,
    minFrequency: 2,
    showProgress: true,
    specialTokens: ["<unk>"],
    vocabSize: 30000,
    wordpiecesPrefix: "##"
  };

  private constructor(tokenizer: Tokenizer) {
    super(tokenizer);
  }

  /**
   * Instantiate and returns a new Bert WordPiece tokenizer
   * @param [options] Optional tokenizer options
   */
  static async fromOptions(
    options?: BertWordPieceOptions
  ): Promise<BertWordPieceTokenizer> {
    const opts = { ...this.defaultBertOptions, ...options };

    let model: Model;
    if (opts.vocabFile) {
      // const fromFiles = promisify(WordPiece.fromFiles);
      model = WordPiece.fromFiles(opts.vocabFile, { unkToken: opts.unkToken });
      // model = await fromFiles(mergedOptions.vocabFile, mergedOptions.unkToken, null);
    } else {
      model = WordPiece.empty();
    }

    const tokenizer = new Tokenizer(model);

    const normalizer = bertNormalizer(opts);
    tokenizer.setNormalizer(normalizer);
    tokenizer.setPreTokenizer(bertPreTokenizer());

    if (opts.vocabFile && opts.addSpecialTokens) {
      const sepTokenId = tokenizer.tokenToId(opts.sepToken);
      if (sepTokenId === undefined) {
        throw new Error("sepToken not found in the vocabulary");
      }

      const clsTokenId = tokenizer.tokenToId(opts.clsToken);
      if (clsTokenId === undefined) {
        throw new Error("clsToken not found in the vocabulary");
      }

      if (opts.addSpecialTokens) {
        const processor = bertProcessing(
          [opts.sepToken, sepTokenId],
          [opts.clsToken, clsTokenId]
        );
        tokenizer.setPostProcessor(processor);
      }
    }

    const decoder = wordPieceDecoder(opts.wordpiecesPrefix);
    tokenizer.setDecoder(decoder);

    return new BertWordPieceTokenizer(tokenizer);
  }

  /**
   * Train the model using the given files
   *
   * @param files Files to use for training
   * @param [options] Training options
   */
  async train(files: string[], options?: BertWordPieceTrainOptions): Promise<void> {
    const mergedOptions = { ...this.defaultTrainOptions, ...options };
    const trainer = wordPieceTrainer(mergedOptions);

    this.tokenizer.train(trainer, files);
  }
}

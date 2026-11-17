/**
 * Pinboard Inline Math Calculator Parser
 *
 * Safe, zero-eval recursive descent mathematical expression parser.
 * Supports +, -, *, /, %, ^, parentheses, negative numbers, and decimals.
 */

export interface CalculationResult {
  success: boolean;
  result?: number;
  formatted?: string;
  error?: string;
}

export class MathParser {
  private pos = 0;
  private expr = '';

  constructor(expr: string) {
    // Clean expression: remove spaces and normalize commas inside numbers (e.g. "1,000" -> "1000")
    // Keep commas that aren't between digits
    this.expr = expr
      .replace(/(\d),(\d)/g, '$1$2')
      .replace(/\s+/g, '');
  }

  evaluate(): CalculationResult {
    if (!this.expr) {
      return { success: false, error: 'Empty expression' };
    }

    // Safety check: only allow digits, arithmetic operators, parentheses, and dots
    if (/[^0-9+\-*/%^().]/.test(this.expr)) {
      const invalidChars = this.expr.replace(/[0-9+\-*/%^().]/g, '');
      return { success: false, error: `Invalid character: ${invalidChars[0]}` };
    }

    try {
      this.pos = 0;
      const res = this.parseExpression();

      if (this.pos < this.expr.length) {
        return { success: false, error: `Unexpected '${this.expr[this.pos]}'` };
      }

      if (!isFinite(res)) {
        return { success: false, error: 'Division by zero' };
      }

      return {
        success: true,
        result: res,
        formatted: this.formatNumber(res),
      };
    } catch (e: any) {
      return { success: false, error: e.message || 'Invalid expression' };
    }
  }

  private parseExpression(): number {
    let value = this.parseTerm();
    while (this.pos < this.expr.length) {
      const op = this.expr[this.pos];
      if (op === '+' || op === '-') {
        this.pos++;
        const nextTerm = this.parseTerm();
        value = op === '+' ? value + nextTerm : value - nextTerm;
      } else {
        break;
      }
    }
    return value;
  }

  private parseTerm(): number {
    let value = this.parseFactor();
    while (this.pos < this.expr.length) {
      const op = this.expr[this.pos];
      if (op === '*' || op === '/' || op === '%') {
        this.pos++;
        const nextFactor = this.parseFactor();
        if (op === '/') {
          if (nextFactor === 0) throw new Error('Division by zero');
          value = value / nextFactor;
        } else if (op === '%') {
          if (nextFactor === 0) throw new Error('Division by zero');
          value = value % nextFactor;
        } else {
          value = value * nextFactor;
        }
      } else {
        break;
      }
    }
    return value;
  }

  private parseFactor(): number {
    let base = this.parseUnary();
    if (this.pos < this.expr.length && this.expr[this.pos] === '^') {
      this.pos++;
      const exponent = this.parseFactor(); // Right-associative exponentiation
      base = Math.pow(base, exponent);
    }
    return base;
  }

  private parseUnary(): number {
    if (this.pos < this.expr.length && this.expr[this.pos] === '+') {
      this.pos++;
      return this.parseUnary();
    }
    if (this.pos < this.expr.length && this.expr[this.pos] === '-') {
      this.pos++;
      return -this.parseUnary();
    }
    return this.parsePrimary();
  }

  private parsePrimary(): number {
    if (this.pos >= this.expr.length) {
      throw new Error('Unexpected end of expression');
    }

    if (this.expr[this.pos] === '(') {
      this.pos++;
      const val = this.parseExpression();
      if (this.pos >= this.expr.length || this.expr[this.pos] !== ')') {
        throw new Error('Missing closing parenthesis');
      }
      this.pos++; // skip ')'
      return val;
    }

    // Number parsing
    const start = this.pos;
    let hasDot = false;
    while (this.pos < this.expr.length) {
      const char = this.expr[this.pos];
      if (char >= '0' && char <= '9') {
        this.pos++;
      } else if (char === '.' && !hasDot) {
        hasDot = true;
        this.pos++;
      } else {
        break;
      }
    }

    if (start === this.pos) {
      throw new Error(`Expected number at '${this.expr.slice(this.pos, this.pos + 3)}'`);
    }

    const numStr = this.expr.slice(start, this.pos);
    const num = parseFloat(numStr);
    if (isNaN(num)) {
      throw new Error(`Invalid number '${numStr}'`);
    }
    return num;
  }

  private formatNumber(num: number): string {
    if (Number.isInteger(num)) {
      return num.toLocaleString('en-US');
    }
    // Round to 6 decimal places to prevent floating point inaccuracies like 0.30000000000000004
    const rounded = Number(num.toFixed(6));
    const parts = rounded.toString().split('.');
    const intFormatted = parseInt(parts[0], 10).toLocaleString('en-US');
    return parts.length > 1 ? `${intFormatted}.${parts[1]}` : intFormatted;
  }
}

/**
 * Evaluates a mathematical expression safely without using eval() or new Function().
 */
export function evaluateFormula(expression: string): CalculationResult {
  const clean = expression.replace(/^[ \t]*=[ \t]*/, '').trim();
  const parser = new MathParser(clean);
  return parser.evaluate();
}

/**
 * Checks if a line is a standalone formula line (e.g. "= 5 + 5" or "=5+5").
 */
export function isFormulaLine(line: string): boolean {
  return /^[ \t]*=[ \t]*\S/.test(line);
}

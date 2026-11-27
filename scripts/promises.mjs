import { existsSync, readdirSync, realpathSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript-api";

const FLOATING =
  "Promises must be awaited, end with a call to .catch, end with a call to .then with a rejection handler or be explicitly marked as ignored with the `void` operator.";
const NOT_A_FUNCTION = " A rejection handler that is not a function will be ignored.";
const ARRAY =
  "An array of Promises may be unintentional. Consider handling the promises' fulfillment or rejection with Promise.all or similar, or explicitly marking the expression as ignored with the `void` operator.";

const libFolder = resolve(dirname(fileURLToPath(import.meta.resolve("typescript-api"))));

const projectsOf = (root) => {
  const packages = existsSync(join(root, "packages")) ? readdirSync(join(root, "packages")) : [];
  return [join(root, "app", "tsconfig.json"), ...packages.map((one) => join(root, "packages", one, "tsconfig.json"))].filter(
    (path) => existsSync(path),
  );
};

const scanned = (root, file) => {
  const path = relative(root, file).split(sep).join("/");
  if (path.endsWith(".d.ts") || path.startsWith("app/src/tests/")) return null;
  return /^(app\/src|packages\/[^/]+\/src)\//.test(path) ? path : null;
};

const withoutPackages = (options) => {
  const host = ts.createCompilerHost(options);
  const outside = (path) => {
    const at = resolve(path);
    return at.split(sep).includes("node_modules") && !at.startsWith(libFolder);
  };
  const fileExists = host.fileExists.bind(host);
  const directoryExists = host.directoryExists?.bind(host);
  const readFile = host.readFile.bind(host);
  host.fileExists = (path) => !outside(path) && fileExists(path);
  host.directoryExists = (path) => !outside(path) && (directoryExists ? directoryExists(path) : true);
  host.readFile = (path) => (outside(path) ? undefined : readFile(path));
  return host;
};

const programOf = (config) => {
  const parsed = ts.getParsedCommandLineOfConfigFile(config, {}, { ...ts.sys, onUnRecoverableConfigFileDiagnostic: () => {} });
  if (!parsed) throw new Error(`${config} could not be read`);
  return ts.createProgram({ rootNames: parsed.fileNames, options: parsed.options, host: withoutPackages(parsed.options) });
};

const unwrap = (node) => (ts.isParenthesizedExpression(node) ? unwrap(node.expression) : node);

const methodOf = (call) => {
  const callee = unwrap(call.expression);
  if (ts.isPropertyAccessExpression(callee)) return { name: callee.name.text, object: callee.expression };
  if (ts.isElementAccessExpression(callee) && ts.isStringLiteralLike(callee.argumentExpression))
    return { name: callee.argumentExpression.text, object: callee.expression };
  return null;
};

const isAssignment = (kind) => kind >= ts.SyntaxKind.FirstAssignment && kind <= ts.SyntaxKind.LastAssignment;
const isLogical = (kind) =>
  kind === ts.SyntaxKind.AmpersandAmpersandToken ||
  kind === ts.SyntaxKind.BarBarToken ||
  kind === ts.SyntaxKind.QuestionQuestionToken;

const judge = (program) => {
  const checker = program.getTypeChecker();

  const fromDefaultLibrary = (symbol) =>
    (symbol.getDeclarations() ?? []).some((one) => program.isSourceFileDefaultLibrary(one.getSourceFile()));

  const isBuiltinPromise = (type) => {
    if (type.isIntersection()) return type.types.some(isBuiltinPromise);
    if (type.isUnion()) return type.types.every(isBuiltinPromise);
    if (type.flags & ts.TypeFlags.TypeParameter) {
      const constraint = type.getConstraint();
      return constraint ? isBuiltinPromise(constraint) : false;
    }
    const symbol = type.getSymbol();
    if (!symbol) return false;
    if (symbol.getName() === "Promise" && fromDefaultLibrary(symbol)) return true;
    if (symbol.flags & (ts.SymbolFlags.Class | ts.SymbolFlags.Interface))
      return (checker.getBaseTypes(type) ?? []).some(isBuiltinPromise);
    return false;
  };

  const partsOf = (type) => (type.isUnion() ? type.types : [type]);
  const isPromiseLike = (type) => partsOf(checker.getApparentType(type)).some(isBuiltinPromise);

  const isPromiseArray = (node) =>
    partsOf(checker.getTypeAtLocation(node))
      .map((one) => checker.getApparentType(one))
      .some((one) => {
        if (checker.isArrayType(one)) return isPromiseLike(checker.getTypeArguments(one)[0]);
        if (checker.isTupleType(one)) return checker.getTypeArguments(one).some(isPromiseLike);
        return false;
      });

  const handles = (handler) => checker.getTypeAtLocation(handler).getCallSignatures().length > 0;
  const HANDLED = { unhandled: false };

  const verdict = (raw) => {
    const node = unwrap(raw);
    if (ts.isBinaryExpression(node) && isAssignment(node.operatorToken.kind)) return HANDLED;
    if (ts.isBinaryExpression(node) && node.operatorToken.kind === ts.SyntaxKind.CommaToken) {
      const left = verdict(node.left);
      return left.unhandled ? left : verdict(node.right);
    }
    if (isPromiseArray(node)) return { unhandled: true, message: ARRAY };
    if (ts.isAwaitExpression(node)) return HANDLED;
    if (!isPromiseLike(checker.getTypeAtLocation(node))) return HANDLED;
    if (ts.isCallExpression(node)) {
      const method = methodOf(node);
      const handler =
        method?.name === "catch" && node.arguments.length >= 1
          ? node.arguments[0]
          : method?.name === "then" && node.arguments.length >= 2
            ? node.arguments[1]
            : null;
      if (handler) return handles(handler) ? HANDLED : { unhandled: true, message: FLOATING + NOT_A_FUNCTION };
      if (method?.name === "finally") return verdict(method.object);
      return { unhandled: true, message: FLOATING };
    }
    if (ts.isConditionalExpression(node)) {
      const otherwise = verdict(node.whenFalse);
      return otherwise.unhandled ? otherwise : verdict(node.whenTrue);
    }
    if (ts.isBinaryExpression(node) && isLogical(node.operatorToken.kind)) {
      const left = verdict(node.left);
      return left.unhandled ? left : verdict(node.right);
    }
    return { unhandled: true, message: FLOATING };
  };

  return verdict;
};

export const floating = (root) => {
  const found = new Map();
  const projects = projectsOf(resolve(root));
  if (projects.length === 0) throw new Error(`${resolve(root)} has no app/tsconfig.json to read`);
  for (const config of projects) {
    const program = programOf(config);
    const verdict = judge(program);
    for (const source of program.getSourceFiles()) {
      const path = scanned(resolve(root), source.fileName);
      if (!path) continue;
      const visit = (node) => {
        if (ts.isExpressionStatement(node)) {
          const { unhandled, message } = verdict(node.expression);
          if (unhandled) {
            const { line, character } = source.getLineAndCharacterOfPosition(node.getStart(source));
            const at = `${path}:${line + 1}:${character + 1}`;
            found.set(at, { file: path, line: line + 1, column: character + 1, message });
          }
        }
        ts.forEachChild(node, visit);
      };
      visit(source);
    }
  }
  return [...found.values()].sort((a, b) => a.file.localeCompare(b.file) || a.line - b.line);
};

if (process.argv[1] && realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url))) {
  const root = process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), "..");
  const hits = floating(root);
  for (const one of hits) console.log(`${one.file}:${one.line}:${one.column} ${one.message}`);
  if (hits.length > 0) {
    console.error(`\n${hits.length} promise(s) left floating: Sonar's typescript:S9383 refuses each one`);
    process.exitCode = 1;
  }
}

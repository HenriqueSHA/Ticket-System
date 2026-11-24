# 19. Conclusão

O projeto partiu de um problema real: a gestão de eventos com processos manuais e pouco integrados,
que causa venda duplicada, falta de controle dos assentos e dados desorganizados. Para resolvê-lo, a
equipe dividiu o sistema em módulos (cadastros, consulta, compra, cancelamento e remarcação,
relatórios e administração). Cada módulo foi modelado com entradas, regras e saídas e descrito em
pseudocódigo e fluxogramas, e só depois foi programado.

A solução implementada atende ao objetivo geral e a todos os objetivos específicos. Ela centraliza
organizadores, eventos, categorias, assentos, clientes e vendas em uma única plataforma web, com três
perfis de acesso, e impede a venda duplicada com validações no programa, transações e restrições no
banco de dados. Os 55 testes automáticos, executados a cada alteração, mostram que os cenários
normais, alternativos e excepcionais se comportam como o esperado.

Durante o desenvolvimento foram aplicados os conteúdos da disciplina:

- estruturas sequenciais, condicionais e de repetição nos algoritmos;
- vetores (dígitos do CPF), matrizes (mapa de assentos), listas e registros;
- modularização em sub-algoritmos;
- validação de entradas e tratamento de erros;
- orientação a objetos com encapsulamento, abstração e polimorfismo, além de princípios SOLID e
  padrões de projeto.

A equipe também praticou persistência com SQL e transações, testes automatizados, versionamento com
Git e entrega contínua com contêineres. O resultado é um sistema funcional, documentado e pronto para
evoluir com as melhorias da seção 18.

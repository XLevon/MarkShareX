# Agentic Engineering 日益成熟：从 Prompt 到 Harness 的三层进化

> **TL;DR**：Prompt Engineering 是"写好一句话"，Context Engineering 是"组装好一次输入"，Harness Engineering 是"设计好一整套让 Agent 稳定运行的环境"。三者是 2022-2025 年逐层叠加的演进路径——做 ChatBot 只用第一层，做 RAG 助手要加第二层，做 Claude Code/Cursor 这类 Agent 三层都得做。

---

![Agentic Engineering 三层架构](/uploads/agentic-engineering-layers.svg)

---

## 一、为什么要理解这三层？

AI Agent 在 2025 年进入了爆发期。Claude Code、Cursor Composer、OpenHands 等产品展示了惊人的自主编程能力。但你有没有想过一个关键问题：

> **为什么同一个底层模型（比如 Claude 4），在 Cursor 里表现惊艳，在自己写的 LangChain 脚本里却频频翻车？**

答案就在这**三个工程的差距**里。Anthropic 在 SWE-bench 论文中明确指出：**Agent 的性能差异，很大程度上来自 scaffolding（脚手架）质量的不同，而不是模型本身。**

---

## 二、三层概念详解

### 2.1 Prompt Engineering — 提示工程（让模型"听懂"）

**一句话定义**：设计单次给模型的输入指令，让模型按预期方向输出。

**核心问题**：模型听不听得懂你在说什么？

**起源**：2022 年 GPT-3 时代，人们发现同样的问题换一种问法，输出质量天差地别。

**典型做法**：
- **角色定位**：明确你是什么角色、模型扮演什么角色
- **任务分解**：把复杂任务拆成简单步骤，逐个引导
- **输出格式约束**：指定 JSON、Markdown 等结构化输出格式
- **少样本示例**：给出 2-3 个高质量示例引导模型行为
- **边界约束**：明确"什么不要做"往往比"要做什么"更重要

**实例对比**：

| 模糊 Prompt | 精确 Prompt |
|------------|------------|
| "帮我优化这段代码" | "保留原有结构和变量命名，只优化时间复杂度，不要改返回值格式" |
| 结果：可能全改 | 结果：精准命中 |

**局限**：Prompt Engineering 只能影响"怎么说"，不能改变"模型能做什么"，也无法解决跨 session 的连贯性问题。当任务从"生成一段文字"变成"完成一个复杂的多步骤任务"，单靠 prompt 设计远远不够。

---

### 2.2 Context Engineering — 上下文工程（让模型"看到该看的"）

**一句话定义**：在每次 LLM 推理时，策展出最优的 token 集合——在正确时机塞入正确信息。

**核心问题**：模型手里有没有完成任务需要的全部信息？

**起源**：Andrej Karpathy 于 2025 年正式提出，Anthropic 将其系统化定义为七大组件。

**上下文七大组件**：

```
1. System Prompt        — 系统级指令
2. User Prompt          — 用户输入
3. State & History      — 对话状态与历史
4. Long-Term Memory     — 长期记忆
5. Retrieved Info(RAG)  — 检索增强信息
6. Available Tools      — 可用工具定义
7. Structured Output    — 结构化输出格式
```

**核心原则**（Anthropic 官方）：
> 找到"the smallest possible set of high-signal tokens that maximize the likelihood of some desired outcome"——用最少的高信号 token 达成目标。

**关键认知**：
- **不是越多越好**：上下文窗口再大（200K+），信息一多模型就开始"失忆"——前面的约束被冲淡，目标逐渐漂移
- **三大动作**：召回 → 压缩 → 组装
  - **召回**：从海量信息中提取与当前任务最相关内容
  - **压缩**：将过长文档、日志、历史对话转化为摘要和关键点
  - **组装**：按优先级排列上下文，关键指令放在模型最容易注意的位置

**Philipp Schmid 点睛**：
> "Agent failures aren't only model failures; they are context failures."

---

### 2.3 Harness Engineering — 执行环境工程（让模型"稳定干成"）

**一句话定义**：围绕大模型构建的一整套执行与控制系统——给模型装上手脚、规则、记忆、反馈和约束。

**核心问题**：当模型在真实环境中连续行动时，系统怎么保证它不跑偏、不崩溃？出了错还能拉回来？

**起源**：Anthropic 在 SWE-bench 论文中将 Agent 定义为 **"Agent = Model + Harness"**。OpenAI 内部用三个工程师 + AI Agent 花 5 个月写了一个百万行代码的产品，人类手写代码量为零——全程不碰代码，只给 AI 搭"鹰架"。

**六层核心架构**：

| 层级 | 功能 | 例子 |
|------|------|------|
| 1. 结构化上下文管理 | 明确角色、目标、成功标准，过滤无关信息 | CLAUDE.md 项目规范文件 |
| 2. 工具系统设计 | 模型接工具获得行动力；关键在何时调用、调用结果如何反馈 | Read/Edit/Write/Bash/WebFetch |
| 3. 执行编排引擎 | 复杂任务轨道：理解任务→找信息缺口→调工具→产出→检查→循环 | ReAct / Plan-Execute 模式 |
| 4. 状态与记忆管理 | 区分当前进度、中间产物、长期记忆，防止越跑越乱 | /compact + checkpointing |
| 5. 独立评估与观测 | 内置评估机制，检查输出质量、记录日志、统计错误率 | Evaluator Agent 独立验收 |
| 6. 约束校验与恢复 | 定义边界条件、关键节点校验、失败回滚/重试/换方案 | 沙箱隔离 + 权限分级 |

**生活化比喻**：
- Prompt Engineering = 给实习生布置任务时，尽量把话说清楚
- Context Engineering = 提前准备好相关资料、客户背景、模板文档
- Harness Engineering = 加上检查清单、汇报机制、阶段验收、错误回滚、会后复盘

---

## 三、演进时间线与代表产品

| 年份 | 主流范式 | 代表产品 | 工程焦点 |
|------|----------|----------|----------|
| 2022 | Prompt Engineering | ChatGPT / Jasper / Copy.ai | 怎么写 prompt |
| 2023 | + RAG | LangChain / LlamaIndex | 怎么塞知识进 prompt |
| 2024 | Context Engineering | Cursor / Devin / Dify | 怎么管理 token 预算 |
| 2025 | Harness Engineering | Claude Code / Cursor Composer / OpenHands | 怎么设计 Agent 跑得稳 |

**演进驱动力**：模型能力增强 → 上下文窗口扩大（200K+）→ 外部工具增多（MCP 生态）→ "塞什么、什么时候塞、塞完怎么用"变得比"怎么写"重要得多。

---

## 四、实战：各层最佳实践

### 4.1 Prompt Engineering 六大技巧

```yaml
# 示例：一个精心设计的 System Prompt
角色: 你是一个资深 Go 后端开发工程师
任务: 审查以下代码的安全漏洞
输出格式: 
  - 按严重程度排序（Critical > High > Medium > Low）
  - 每条包含：文件位置 + 问题描述 + 修复建议 + 代码示例
边界约束:
  - 不要修改代码风格（缩进、命名）
  - 不要添加新功能
  - 如果代码没有安全问题，直接回复"未发现安全问题"
```

### 4.2 Context Engineering 七大技巧

| 技巧 | 说明 |
|------|------|
| System Prompt 精校 | 每一行问自己"删掉这行，模型会不会犯错？" |
| Token-efficient tools | 工具描述简短、避免功能重叠 |
| 策展示例 | 用 2-3 个典型示例代替长篇规则 |
| Just-in-time 检索 | 不预加载所有文档，按需 runtime 取 |
| Compaction | 长任务做摘要压缩，防止上下文过载 |
| 结构化笔记 | 关键信息写入 memory 文件，而非塞回上下文 |
| Sub-agent 架构 | 子 Agent 在独立上下文里做研究，只回传结论 |

### 4.3 Harness Engineering 八层架构参考（Claude Code 模式）

```
采样循环：while(not_done): call_model → parse → execute → append_result
工具系统：Read / Edit / Write / Bash / WebFetch / Grep / Glob
权限层：  Auto mode / Allowlist / Sandboxing 三档可调
上下文管理器：/clear、/compact、checkpointing、rewind
记忆系统：CLAUDE.md（项目级）+ 用户级 memory 文件
子Agent编排：Task tool 派生子上下文并行工作
钩子机制：工具调用前后自动触发 lint / 测试
错误恢复：工具失败重试、上下文超限自动压缩
```

---

## 五、真实案例数据

### 案例一：Harness 改造将成功率从 60% 提升到 90%

某团队几乎不动模型，仅通过改进任务拆解、状态管理、结果校验和反馈闭环，就将 Agent 任务成功率从 **60% 提升到 90%**。**同样模型、同样提示词，只要 Harness 设计不同，表现天差地别。**

### 案例二：生产与验收分离

让 Agent 自己给自己打分，几乎总是过于乐观。最佳实践是将生产与验收彻底分离：
- **Producer Agent**：负责实现功能
- **Evaluator Agent**：独立检查——跑界面、看日志、验逻辑，像 QA 一样

### 案例三：长任务重启策略

很多长任务失败，不是因为模型不够强，而是系统没有及时重置。过长的上下文导致模型焦虑、遗忘、偷懒、急着收尾。激进策略：**任务过载时直接重启一个新的 Agent 实例，只交接关键状态**——像程序进程崩溃时重启，而非死命清理缓存。

---

## 六、应用建议：你该投入哪一层？

| 你在做什么 | 必做层 | 选做层 |
|------------|--------|--------|
| 写一个客服 ChatBot | Prompt | — |
| 做一个文档问答助手 | Prompt + Context (RAG) | — |
| 做一个 Agent 类产品（Coding/研究/自动化） | Prompt + Context + **Harness** | — |
| 接入 Claude Code / Cursor 使用 | 看懂 Harness、写好 CLAUDE.md | — |

---

## 七、常见误区

❌ **误区一："Context Engineering 取代了 Prompt Engineering"**
✅ 真相：Prompt 是 Context 的子集。写不好 prompt，塞再多上下文也没用。

❌ **误区二："Harness 就是 Agent 框架（LangGraph / AutoGen）"**
✅ 真相：框架只是实现工具，Harness 是一套设计哲学——框架可以换，哲学不能丢。

❌ **误区三："RAG = Context Engineering"**
✅ 真相：RAG 只是 Context Engineering 七大组件之一。真正的上下文工程还包括 system prompt 精校、工具 token 预算管理、历史 compaction、sub-agent 架构等。

❌ **误区四："模型越强，Harness 就越不重要"**
✅ 真相：恰恰相反。模型越强，能做的事情越多，越需要 Harness 来约束、引导和兜底。

---

## 八、总结

> **模型像 CPU 负责算，而 Harness 更像操作系统，负责调度、内存、IO、约束、恢复和反馈。没有操作系统，再强的 CPU 也只是一个裸奔的计算单元。**

用一句话记住全部：
> **Prompt 是说话，Context 是给材料，Harness 是保证干成。**

三者层层嵌套，缺一不可。AI 工程的核心正在从"让模型看起来聪明"转向 **"让模型真正稳定地工作"**。

---

**延伸阅读**：
- Anthropic 官方：Context Engineering 指南
- Anthropic SWE-bench 论文：Agent = Model + Harness
- Claude Code 架构：Harness Engineering 的工业级参考实现

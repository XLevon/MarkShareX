import{a9 as N,cm as yn,co as wn,aA as he,ci as Dn,by as h,cR as go,cu as cn,cK as E,dV as po,cW as Cn,e4 as _e,cs as Hn,bs as un,d4 as fn,cx as bo,b_ as mo,bR as Kn,t as yo,bQ as He,w as Ct,ds as Ke,M as $n,c as wo,j as Jn,aT as Co,U as Qn,bU as et,h as rn,c3 as xo,bZ as nt,bY as So,bW as Oo,bV as Po,bP as Ro,bI as ko,v as To,u as _o,H as A,O as D,G as Q,d as xt,dK as Ae,dX as ce,dY as We,aS as Mo,al as ee,cO as Le,bJ as Wn,dP as Ze,l as jn,P as X,Q as xe,aX as St,c$ as $e,e as zo,S as $o,cZ as Fo,dT as hn,dn as te,bN as Io,aL as Ao,br as De,cE as Fe,J as Bo,e5 as Vn,c1 as tt,F as Eo,c9 as No,e7 as Un,b as Ot,o as Lo,cB as Do,a0 as ln,e2 as Pt,bu as Fn,cA as Ho,aP as Ko,cg as Wo,bo as jo,a1 as Vo,k as Uo,ea as Go,c4 as qo,bX as Rt,R as de,aM as Zo,a6 as Xo,Y as Z,N as Yo,a3 as ot,ak as Jo,n as Qo,bO as er,dO as nr,c7 as tr,d6 as or,du as rr}from"./index-CI36Xkal.js";import{e as ir,d as lr,f as Gn,b as ar,F as sr,h as Xe,i as dr,V as kt,u as Ie,B as Tt,a as _t}from"./FocusDetector-BFIGNsXQ.js";import{u as Mt,a as In,N as cr}from"./Input-CfEGmlIL.js";import{c as ur,t as qn,i as zt,g as fr,b as hr,f as xn}from"./get-Bwae3wt5.js";function $t(e,n){return N(()=>{for(const t of n)if(e[t]!==void 0)return e[t];return e[n[n.length-1]]})}const Ne="@@mmoContext",vr={mounted(e,{value:n}){e[Ne]={handler:void 0},typeof n=="function"&&(e[Ne].handler=n,wn("mousemoveoutside",e,n))},updated(e,{value:n}){const t=e[Ne];typeof n=="function"?t.handler?t.handler!==n&&(yn("mousemoveoutside",e,t.handler),t.handler=n,wn("mousemoveoutside",e,n)):(e[Ne].handler=n,wn("mousemoveoutside",e,n)):t.handler&&(yn("mousemoveoutside",e,t.handler),t.handler=void 0)},unmounted(e){const{handler:n}=e[Ne];n&&yn("mousemoveoutside",e,n),e[Ne].handler=void 0}},Oe="v-hidden",gr=lr("[v-hidden]",{display:"none!important"}),rt=he({name:"Overflow",props:{getCounter:Function,getTail:Function,updateCounter:Function,onUpdateCount:Function,onUpdateOverflow:Function},setup(e,{slots:n}){const t=E(null),o=E(null);function r(i){const{value:l}=t,{getCounter:u,getTail:f}=e;let d;if(u!==void 0?d=u():d=o.value,!l||!d)return;d.hasAttribute(Oe)&&d.removeAttribute(Oe);const{children:c}=l;if(i.showAllItemsBeforeCalculate)for(const M of c)M.hasAttribute(Oe)&&M.removeAttribute(Oe);const C=l.offsetWidth,O=[],v=n.tail?f==null?void 0:f():null;let g=v?v.offsetWidth:0,k=!1;const x=l.children.length-(n.tail?1:0);for(let M=0;M<x-1;++M){if(M<0)continue;const m=c[M];if(k){m.hasAttribute(Oe)||m.setAttribute(Oe,"");continue}else m.hasAttribute(Oe)&&m.removeAttribute(Oe);const y=m.offsetWidth;if(g+=y,O[M]=y,g>C){const{updateCounter:z}=e;for(let K=M;K>=0;--K){const j=x-1-K;z!==void 0?z(j):d.textContent=`${j}`;const T=d.offsetWidth;if(g-=O[K],g+T<=C||K===0){k=!0,M=K-1,v&&(M===-1?(v.style.maxWidth=`${C-T}px`,v.style.boxSizing="border-box"):v.style.maxWidth="");const{onUpdateCount:F}=e;F&&F(j);break}}}}const{onUpdateOverflow:S}=e;k?S!==void 0&&S(!0):(S!==void 0&&S(!1),d.setAttribute(Oe,""))}const a=po();return gr.mount({id:"vueuc/overflow",head:!0,anchorMetaName:ir,ssr:a}),cn(()=>r({showAllItemsBeforeCalculate:!1})),{selfRef:t,counterRef:o,sync:r}},render(){const{$slots:e}=this;return Dn(()=>this.sync({showAllItemsBeforeCalculate:!1})),h("div",{class:"v-overflow",ref:"selfRef"},[go(e,"default"),e.counter?e.counter():h("span",{style:{display:"inline-block"},ref:"counterRef"}),e.tail?e.tail():null])}});function Ft(e,n){n&&(cn(()=>{const{value:t}=e;t&&Cn.registerHandler(t,n)}),_e(e,(t,o)=>{o&&Cn.unregisterHandler(o)},{deep:!1}),Hn(()=>{const{value:t}=e;t&&Cn.unregisterHandler(t)}))}function Il(e,n){if(!e)return;const t=document.createElement("a");t.href=e,n!==void 0&&(t.download=n),document.body.appendChild(t),t.click(),document.body.removeChild(t)}let Sn;function pr(){return Sn===void 0&&(Sn=navigator.userAgent.includes("Node.js")||navigator.userAgent.includes("jsdom")),Sn}function it(e){switch(typeof e){case"string":return e||void 0;case"number":return String(e);default:return}}function Al(e,n="default",t=[]){const r=e.$slots[n];return r===void 0?t:r()}function On(e){const n=e.filter(t=>t!==void 0);if(n.length!==0)return n.length===1?n[0]:t=>{e.forEach(o=>{o&&o(t)})}}var An=un(fn,"WeakMap"),br=bo(Object.keys,Object),mr=Object.prototype,yr=mr.hasOwnProperty;function wr(e){if(!mo(e))return br(e);var n=[];for(var t in Object(e))yr.call(e,t)&&t!="constructor"&&n.push(t);return n}function Zn(e){return Kn(e)?yo(e):wr(e)}function Cr(e,n){for(var t=-1,o=n.length,r=e.length;++t<o;)e[r+t]=n[t];return e}function xr(e,n){for(var t=-1,o=e==null?0:e.length,r=0,a=[];++t<o;){var i=e[t];n(i,t,e)&&(a[r++]=i)}return a}function Sr(){return[]}var Or=Object.prototype,Pr=Or.propertyIsEnumerable,lt=Object.getOwnPropertySymbols,Rr=lt?function(e){return e==null?[]:(e=Object(e),xr(lt(e),function(n){return Pr.call(e,n)}))}:Sr;function kr(e,n,t){var o=n(e);return He(e)?o:Cr(o,t(e))}function at(e){return kr(e,Zn,Rr)}var Bn=un(fn,"DataView"),En=un(fn,"Promise"),Nn=un(fn,"Set"),st="[object Map]",Tr="[object Object]",dt="[object Promise]",ct="[object Set]",ut="[object WeakMap]",ft="[object DataView]",_r=Ke(Bn),Mr=Ke($n),zr=Ke(En),$r=Ke(Nn),Fr=Ke(An),Te=Ct;(Bn&&Te(new Bn(new ArrayBuffer(1)))!=ft||$n&&Te(new $n)!=st||En&&Te(En.resolve())!=dt||Nn&&Te(new Nn)!=ct||An&&Te(new An)!=ut)&&(Te=function(e){var n=Ct(e),t=n==Tr?e.constructor:void 0,o=t?Ke(t):"";if(o)switch(o){case _r:return ft;case Mr:return st;case zr:return dt;case $r:return ct;case Fr:return ut}return n});var Ir="__lodash_hash_undefined__";function Ar(e){return this.__data__.set(e,Ir),this}function Br(e){return this.__data__.has(e)}function an(e){var n=-1,t=e==null?0:e.length;for(this.__data__=new wo;++n<t;)this.add(e[n])}an.prototype.add=an.prototype.push=Ar;an.prototype.has=Br;function Er(e,n){for(var t=-1,o=e==null?0:e.length;++t<o;)if(n(e[t],t,e))return!0;return!1}function Nr(e,n){return e.has(n)}var Lr=1,Dr=2;function It(e,n,t,o,r,a){var i=t&Lr,l=e.length,u=n.length;if(l!=u&&!(i&&u>l))return!1;var f=a.get(e),d=a.get(n);if(f&&d)return f==n&&d==e;var c=-1,C=!0,O=t&Dr?new an:void 0;for(a.set(e,n),a.set(n,e);++c<l;){var v=e[c],g=n[c];if(o)var k=i?o(g,v,c,n,e,a):o(v,g,c,e,n,a);if(k!==void 0){if(k)continue;C=!1;break}if(O){if(!Er(n,function(x,S){if(!Nr(O,S)&&(v===x||r(v,x,t,o,a)))return O.push(S)})){C=!1;break}}else if(!(v===g||r(v,g,t,o,a))){C=!1;break}}return a.delete(e),a.delete(n),C}function Hr(e){var n=-1,t=Array(e.size);return e.forEach(function(o,r){t[++n]=[r,o]}),t}function Kr(e){var n=-1,t=Array(e.size);return e.forEach(function(o){t[++n]=o}),t}var Wr=1,jr=2,Vr="[object Boolean]",Ur="[object Date]",Gr="[object Error]",qr="[object Map]",Zr="[object Number]",Xr="[object RegExp]",Yr="[object Set]",Jr="[object String]",Qr="[object Symbol]",ei="[object ArrayBuffer]",ni="[object DataView]",ht=Jn?Jn.prototype:void 0,Pn=ht?ht.valueOf:void 0;function ti(e,n,t,o,r,a,i){switch(t){case ni:if(e.byteLength!=n.byteLength||e.byteOffset!=n.byteOffset)return!1;e=e.buffer,n=n.buffer;case ei:return!(e.byteLength!=n.byteLength||!a(new Qn(e),new Qn(n)));case Vr:case Ur:case Zr:return Co(+e,+n);case Gr:return e.name==n.name&&e.message==n.message;case Xr:case Jr:return e==n+"";case qr:var l=Hr;case Yr:var u=o&Wr;if(l||(l=Kr),e.size!=n.size&&!u)return!1;var f=i.get(e);if(f)return f==n;o|=jr,i.set(e,n);var d=It(l(e),l(n),o,r,a,i);return i.delete(e),d;case Qr:if(Pn)return Pn.call(e)==Pn.call(n)}return!1}var oi=1,ri=Object.prototype,ii=ri.hasOwnProperty;function li(e,n,t,o,r,a){var i=t&oi,l=at(e),u=l.length,f=at(n),d=f.length;if(u!=d&&!i)return!1;for(var c=u;c--;){var C=l[c];if(!(i?C in n:ii.call(n,C)))return!1}var O=a.get(e),v=a.get(n);if(O&&v)return O==n&&v==e;var g=!0;a.set(e,n),a.set(n,e);for(var k=i;++c<u;){C=l[c];var x=e[C],S=n[C];if(o)var M=i?o(S,x,C,n,e,a):o(x,S,C,e,n,a);if(!(M===void 0?x===S||r(x,S,t,o,a):M)){g=!1;break}k||(k=C=="constructor")}if(g&&!k){var m=e.constructor,y=n.constructor;m!=y&&"constructor"in e&&"constructor"in n&&!(typeof m=="function"&&m instanceof m&&typeof y=="function"&&y instanceof y)&&(g=!1)}return a.delete(e),a.delete(n),g}var ai=1,vt="[object Arguments]",gt="[object Array]",on="[object Object]",si=Object.prototype,pt=si.hasOwnProperty;function di(e,n,t,o,r,a){var i=He(e),l=He(n),u=i?gt:Te(e),f=l?gt:Te(n);u=u==vt?on:u,f=f==vt?on:f;var d=u==on,c=f==on,C=u==f;if(C&&et(e)){if(!et(n))return!1;i=!0,d=!1}if(C&&!d)return a||(a=new rn),i||xo(e)?It(e,n,t,o,r,a):ti(e,n,u,t,o,r,a);if(!(t&ai)){var O=d&&pt.call(e,"__wrapped__"),v=c&&pt.call(n,"__wrapped__");if(O||v){var g=O?e.value():e,k=v?n.value():n;return a||(a=new rn),r(g,k,t,o,a)}}return C?(a||(a=new rn),li(e,n,t,o,r,a)):!1}function Xn(e,n,t,o,r){return e===n?!0:e==null||n==null||!nt(e)&&!nt(n)?e!==e&&n!==n:di(e,n,t,o,Xn,r)}var ci=1,ui=2;function fi(e,n,t,o){var r=t.length,a=r;if(e==null)return!a;for(e=Object(e);r--;){var i=t[r];if(i[2]?i[1]!==e[i[0]]:!(i[0]in e))return!1}for(;++r<a;){i=t[r];var l=i[0],u=e[l],f=i[1];if(i[2]){if(u===void 0&&!(l in e))return!1}else{var d=new rn,c;if(!(c===void 0?Xn(f,u,ci|ui,o,d):c))return!1}}return!0}function At(e){return e===e&&!So(e)}function hi(e){for(var n=Zn(e),t=n.length;t--;){var o=n[t],r=e[o];n[t]=[o,r,At(r)]}return n}function Bt(e,n){return function(t){return t==null?!1:t[e]===n&&(n!==void 0||e in Object(t))}}function vi(e){var n=hi(e);return n.length==1&&n[0][2]?Bt(n[0][0],n[0][1]):function(t){return t===e||fi(t,e,n)}}function gi(e,n){return e!=null&&n in Object(e)}function pi(e,n,t){n=ur(n,e);for(var o=-1,r=n.length,a=!1;++o<r;){var i=qn(n[o]);if(!(a=e!=null&&t(e,i)))break;e=e[i]}return a||++o!=r?a:(r=e==null?0:e.length,!!r&&Oo(r)&&Po(i,r)&&(He(e)||Ro(e)))}function bi(e,n){return e!=null&&pi(e,n,gi)}var mi=1,yi=2;function wi(e,n){return zt(e)&&At(n)?Bt(qn(e),n):function(t){var o=fr(t,e);return o===void 0&&o===n?bi(t,e):Xn(n,o,mi|yi)}}function Ci(e){return function(n){return n==null?void 0:n[e]}}function xi(e){return function(n){return hr(n,e)}}function Si(e){return zt(e)?Ci(qn(e)):xi(e)}function Oi(e){return typeof e=="function"?e:e==null?ko:typeof e=="object"?He(e)?wi(e[0],e[1]):vi(e):Si(e)}function Pi(e,n){return e&&To(e,n,Zn)}function Ri(e,n){return function(t,o){if(t==null)return t;if(!Kn(t))return e(t,o);for(var r=t.length,a=-1,i=Object(t);++a<r&&o(i[a],a,i)!==!1;);return t}}var ki=Ri(Pi);function Ti(e,n){var t=-1,o=Kn(e)?Array(e.length):[];return ki(e,function(r,a,i){o[++t]=n(r,a,i)}),o}function _i(e,n){var t=He(e)?_o:Ti;return t(e,Oi(n))}const Mi=he({name:"Checkmark",render(){return h("svg",{xmlns:"http://www.w3.org/2000/svg",viewBox:"0 0 16 16"},h("g",{fill:"none"},h("path",{d:"M14.046 3.486a.75.75 0 0 1-.032 1.06l-7.93 7.474a.85.85 0 0 1-1.188-.022l-2.68-2.72a.75.75 0 1 1 1.068-1.053l2.234 2.267l7.468-7.038a.75.75 0 0 1 1.06.032z",fill:"currentColor"})))}}),zi=he({name:"Empty",render(){return h("svg",{viewBox:"0 0 28 28",fill:"none",xmlns:"http://www.w3.org/2000/svg"},h("path",{d:"M26 7.5C26 11.0899 23.0899 14 19.5 14C15.9101 14 13 11.0899 13 7.5C13 3.91015 15.9101 1 19.5 1C23.0899 1 26 3.91015 26 7.5ZM16.8536 4.14645C16.6583 3.95118 16.3417 3.95118 16.1464 4.14645C15.9512 4.34171 15.9512 4.65829 16.1464 4.85355L18.7929 7.5L16.1464 10.1464C15.9512 10.3417 15.9512 10.6583 16.1464 10.8536C16.3417 11.0488 16.6583 11.0488 16.8536 10.8536L19.5 8.20711L22.1464 10.8536C22.3417 11.0488 22.6583 11.0488 22.8536 10.8536C23.0488 10.6583 23.0488 10.3417 22.8536 10.1464L20.2071 7.5L22.8536 4.85355C23.0488 4.65829 23.0488 4.34171 22.8536 4.14645C22.6583 3.95118 22.3417 3.95118 22.1464 4.14645L19.5 6.79289L16.8536 4.14645Z",fill:"currentColor"}),h("path",{d:"M25 22.75V12.5991C24.5572 13.0765 24.053 13.4961 23.5 13.8454V16H17.5L17.3982 16.0068C17.0322 16.0565 16.75 16.3703 16.75 16.75C16.75 18.2688 15.5188 19.5 14 19.5C12.4812 19.5 11.25 18.2688 11.25 16.75L11.2432 16.6482C11.1935 16.2822 10.8797 16 10.5 16H4.5V7.25C4.5 6.2835 5.2835 5.5 6.25 5.5H12.2696C12.4146 4.97463 12.6153 4.47237 12.865 4H6.25C4.45507 4 3 5.45507 3 7.25V22.75C3 24.5449 4.45507 26 6.25 26H21.75C23.5449 26 25 24.5449 25 22.75ZM4.5 22.75V17.5H9.81597L9.85751 17.7041C10.2905 19.5919 11.9808 21 14 21L14.215 20.9947C16.2095 20.8953 17.842 19.4209 18.184 17.5H23.5V22.75C23.5 23.7165 22.7165 24.5 21.75 24.5H6.25C5.2835 24.5 4.5 23.7165 4.5 22.75Z",fill:"currentColor"}))}});function bt(e){return Array.isArray(e)?e:[e]}const Ln={STOP:"STOP"};function Et(e,n){const t=n(e);e.children!==void 0&&t!==Ln.STOP&&e.children.forEach(o=>Et(o,n))}function $i(e,n={}){const{preserveGroup:t=!1}=n,o=[],r=t?i=>{i.isLeaf||(o.push(i.key),a(i.children))}:i=>{i.isLeaf||(i.isGroup||o.push(i.key),a(i.children))};function a(i){i.forEach(r)}return a(e),o}function Fi(e,n){const{isLeaf:t}=e;return t!==void 0?t:!n(e)}function Ii(e){return e.children}function Ai(e){return e.key}function Bi(){return!1}function Ei(e,n){const{isLeaf:t}=e;return!(t===!1&&!Array.isArray(n(e)))}function Ni(e){return e.disabled===!0}function Li(e,n){return e.isLeaf===!1&&!Array.isArray(n(e))}function Rn(e){var n;return e==null?[]:Array.isArray(e)?e:(n=e.checkedKeys)!==null&&n!==void 0?n:[]}function kn(e){var n;return e==null||Array.isArray(e)?[]:(n=e.indeterminateKeys)!==null&&n!==void 0?n:[]}function Di(e,n){const t=new Set(e);return n.forEach(o=>{t.has(o)||t.add(o)}),Array.from(t)}function Hi(e,n){const t=new Set(e);return n.forEach(o=>{t.has(o)&&t.delete(o)}),Array.from(t)}function Ki(e){return(e==null?void 0:e.type)==="group"}function Wi(e){const n=new Map;return e.forEach((t,o)=>{n.set(t.key,o)}),t=>{var o;return(o=n.get(t))!==null&&o!==void 0?o:null}}class ji extends Error{constructor(){super(),this.message="SubtreeNotLoadedError: checking a subtree whose required nodes are not fully loaded."}}function Vi(e,n,t,o){return sn(n.concat(e),t,o,!1)}function Ui(e,n){const t=new Set;return e.forEach(o=>{const r=n.treeNodeMap.get(o);if(r!==void 0){let a=r.parent;for(;a!==null&&!(a.disabled||t.has(a.key));)t.add(a.key),a=a.parent}}),t}function Gi(e,n,t,o){const r=sn(n,t,o,!1),a=sn(e,t,o,!0),i=Ui(e,t),l=[];return r.forEach(u=>{(a.has(u)||i.has(u))&&l.push(u)}),l.forEach(u=>r.delete(u)),r}function Tn(e,n){const{checkedKeys:t,keysToCheck:o,keysToUncheck:r,indeterminateKeys:a,cascade:i,leafOnly:l,checkStrategy:u,allowNotLoaded:f}=e;if(!i)return o!==void 0?{checkedKeys:Di(t,o),indeterminateKeys:Array.from(a)}:r!==void 0?{checkedKeys:Hi(t,r),indeterminateKeys:Array.from(a)}:{checkedKeys:Array.from(t),indeterminateKeys:Array.from(a)};const{levelTreeNodeMap:d}=n;let c;r!==void 0?c=Gi(r,t,n,f):o!==void 0?c=Vi(o,t,n,f):c=sn(t,n,f,!1);const C=u==="parent",O=u==="child"||l,v=c,g=new Set,k=Math.max.apply(null,Array.from(d.keys()));for(let x=k;x>=0;x-=1){const S=x===0,M=d.get(x);for(const m of M){if(m.isLeaf)continue;const{key:y,shallowLoaded:z}=m;if(O&&z&&m.children.forEach(F=>{!F.disabled&&!F.isLeaf&&F.shallowLoaded&&v.has(F.key)&&v.delete(F.key)}),m.disabled||!z)continue;let K=!0,j=!1,T=!0;for(const F of m.children){const W=F.key;if(!F.disabled){if(T&&(T=!1),v.has(W))j=!0;else if(g.has(W)){j=!0,K=!1;break}else if(K=!1,j)break}}K&&!T?(C&&m.children.forEach(F=>{!F.disabled&&v.has(F.key)&&v.delete(F.key)}),v.add(y)):j&&g.add(y),S&&O&&v.has(y)&&v.delete(y)}}return{checkedKeys:Array.from(v),indeterminateKeys:Array.from(g)}}function sn(e,n,t,o){const{treeNodeMap:r,getChildren:a}=n,i=new Set,l=new Set(e);return e.forEach(u=>{const f=r.get(u);f!==void 0&&Et(f,d=>{if(d.disabled)return Ln.STOP;const{key:c}=d;if(!i.has(c)&&(i.add(c),l.add(c),Li(d.rawNode,a))){if(o)return Ln.STOP;if(!t)throw new ji}})}),l}function qi(e,{includeGroup:n=!1,includeSelf:t=!0},o){var r;const a=o.treeNodeMap;let i=e==null?null:(r=a.get(e))!==null&&r!==void 0?r:null;const l={keyPath:[],treeNodePath:[],treeNode:i};if(i!=null&&i.ignored)return l.treeNode=null,l;for(;i;)!i.ignored&&(n||!i.isGroup)&&l.treeNodePath.push(i),i=i.parent;return l.treeNodePath.reverse(),t||l.treeNodePath.pop(),l.keyPath=l.treeNodePath.map(u=>u.key),l}function Zi(e){if(e.length===0)return null;const n=e[0];return n.isGroup||n.ignored||n.disabled?n.getNext():n}function Xi(e,n){const t=e.siblings,o=t.length,{index:r}=e;return n?t[(r+1)%o]:r===t.length-1?null:t[r+1]}function mt(e,n,{loop:t=!1,includeDisabled:o=!1}={}){const r=n==="prev"?Yi:Xi,a={reverse:n==="prev"};let i=!1,l=null;function u(f){if(f!==null){if(f===e){if(!i)i=!0;else if(!e.disabled&&!e.isGroup){l=e;return}}else if((!f.disabled||o)&&!f.ignored&&!f.isGroup){l=f;return}if(f.isGroup){const d=Yn(f,a);d!==null?l=d:u(r(f,t))}else{const d=r(f,!1);if(d!==null)u(d);else{const c=Ji(f);c!=null&&c.isGroup?u(r(c,t)):t&&u(r(f,!0))}}}}return u(e),l}function Yi(e,n){const t=e.siblings,o=t.length,{index:r}=e;return n?t[(r-1+o)%o]:r===0?null:t[r-1]}function Ji(e){return e.parent}function Yn(e,n={}){const{reverse:t=!1}=n,{children:o}=e;if(o){const{length:r}=o,a=t?r-1:0,i=t?-1:r,l=t?-1:1;for(let u=a;u!==i;u+=l){const f=o[u];if(!f.disabled&&!f.ignored)if(f.isGroup){const d=Yn(f,n);if(d!==null)return d}else return f}}return null}const Qi={getChild(){return this.ignored?null:Yn(this)},getParent(){const{parent:e}=this;return e!=null&&e.isGroup?e.getParent():e},getNext(e={}){return mt(this,"next",e)},getPrev(e={}){return mt(this,"prev",e)}};function el(e,n){const t=n?new Set(n):void 0,o=[];function r(a){a.forEach(i=>{o.push(i),!(i.isLeaf||!i.children||i.ignored)&&(i.isGroup||t===void 0||t.has(i.key))&&r(i.children)})}return r(e),o}function nl(e,n){const t=e.key;for(;n;){if(n.key===t)return!0;n=n.parent}return!1}function Nt(e,n,t,o,r,a=null,i=0){const l=[];return e.forEach((u,f)=>{var d;const c=Object.create(o);if(c.rawNode=u,c.siblings=l,c.level=i,c.index=f,c.isFirstChild=f===0,c.isLastChild=f+1===e.length,c.parent=a,!c.ignored){const C=r(u);Array.isArray(C)&&(c.children=Nt(C,n,t,o,r,c,i+1))}l.push(c),n.set(c.key,c),t.has(i)||t.set(i,[]),(d=t.get(i))===null||d===void 0||d.push(c)}),l}function tl(e,n={}){var t;const o=new Map,r=new Map,{getDisabled:a=Ni,getIgnored:i=Bi,getIsGroup:l=Ki,getKey:u=Ai}=n,f=(t=n.getChildren)!==null&&t!==void 0?t:Ii,d=n.ignoreEmptyChildren?m=>{const y=f(m);return Array.isArray(y)?y.length?y:null:y}:f,c=Object.assign({get key(){return u(this.rawNode)},get disabled(){return a(this.rawNode)},get isGroup(){return l(this.rawNode)},get isLeaf(){return Fi(this.rawNode,d)},get shallowLoaded(){return Ei(this.rawNode,d)},get ignored(){return i(this.rawNode)},contains(m){return nl(this,m)}},Qi),C=Nt(e,o,r,c,d);function O(m){if(m==null)return null;const y=o.get(m);return y&&!y.isGroup&&!y.ignored?y:null}function v(m){if(m==null)return null;const y=o.get(m);return y&&!y.ignored?y:null}function g(m,y){const z=v(m);return z?z.getPrev(y):null}function k(m,y){const z=v(m);return z?z.getNext(y):null}function x(m){const y=v(m);return y?y.getParent():null}function S(m){const y=v(m);return y?y.getChild():null}const M={treeNodes:C,treeNodeMap:o,levelTreeNodeMap:r,maxLevel:Math.max(...r.keys()),getChildren:d,getFlattenedNodes(m){return el(C,m)},getNode:O,getPrev:g,getNext:k,getParent:x,getChild:S,getFirstAvailableNode(){return Zi(C)},getPath(m,y={}){return qi(m,y,M)},getCheckedKeys(m,y={}){const{cascade:z=!0,leafOnly:K=!1,checkStrategy:j="all",allowNotLoaded:T=!1}=y;return Tn({checkedKeys:Rn(m),indeterminateKeys:kn(m),cascade:z,leafOnly:K,checkStrategy:j,allowNotLoaded:T},M)},check(m,y,z={}){const{cascade:K=!0,leafOnly:j=!1,checkStrategy:T="all",allowNotLoaded:F=!1}=z;return Tn({checkedKeys:Rn(y),indeterminateKeys:kn(y),keysToCheck:m==null?[]:bt(m),cascade:K,leafOnly:j,checkStrategy:T,allowNotLoaded:F},M)},uncheck(m,y,z={}){const{cascade:K=!0,leafOnly:j=!1,checkStrategy:T="all",allowNotLoaded:F=!1}=z;return Tn({checkedKeys:Rn(y),indeterminateKeys:kn(y),keysToUncheck:m==null?[]:bt(m),cascade:K,leafOnly:j,checkStrategy:T,allowNotLoaded:F},M)},getNonLeafKeys(m={}){return $i(C,m)}};return M}const ol=A("empty",`
 display: flex;
 flex-direction: column;
 align-items: center;
 font-size: var(--n-font-size);
`,[D("icon",`
 width: var(--n-icon-size);
 height: var(--n-icon-size);
 font-size: var(--n-icon-size);
 line-height: var(--n-icon-size);
 color: var(--n-icon-color);
 transition:
 color .3s var(--n-bezier);
 `,[Q("+",[D("description",`
 margin-top: 8px;
 `)])]),D("description",`
 transition: color .3s var(--n-bezier);
 color: var(--n-text-color);
 `),D("extra",`
 text-align: center;
 transition: color .3s var(--n-bezier);
 margin-top: 12px;
 color: var(--n-extra-text-color);
 `)]),rl=Object.assign(Object.assign({},ce.props),{description:String,showDescription:{type:Boolean,default:!0},showIcon:{type:Boolean,default:!0},size:{type:String,default:"medium"},renderIcon:Function}),il=he({name:"Empty",props:rl,slots:Object,setup(e){const{mergedClsPrefixRef:n,inlineThemeDisabled:t,mergedComponentPropsRef:o}=Ae(e),r=ce("Empty","-empty",ol,Mo,e,n),{localeRef:a}=Mt("Empty"),i=N(()=>{var d,c,C;return(d=e.description)!==null&&d!==void 0?d:(C=(c=o==null?void 0:o.value)===null||c===void 0?void 0:c.Empty)===null||C===void 0?void 0:C.description}),l=N(()=>{var d,c;return((c=(d=o==null?void 0:o.value)===null||d===void 0?void 0:d.Empty)===null||c===void 0?void 0:c.renderIcon)||(()=>h(zi,null))}),u=N(()=>{const{size:d}=e,{common:{cubicBezierEaseInOut:c},self:{[ee("iconSize",d)]:C,[ee("fontSize",d)]:O,textColor:v,iconColor:g,extraTextColor:k}}=r.value;return{"--n-icon-size":C,"--n-font-size":O,"--n-bezier":c,"--n-text-color":v,"--n-icon-color":g,"--n-extra-text-color":k}}),f=t?We("empty",N(()=>{let d="";const{size:c}=e;return d+=c[0],d}),u,e):void 0;return{mergedClsPrefix:n,mergedRenderIcon:l,localizedDescription:N(()=>i.value||a.value.description),cssVars:t?void 0:u,themeClass:f==null?void 0:f.themeClass,onRender:f==null?void 0:f.onRender}},render(){const{$slots:e,mergedClsPrefix:n,onRender:t}=this;return t==null||t(),h("div",{class:[`${n}-empty`,this.themeClass],style:this.cssVars},this.showIcon?h("div",{class:`${n}-empty__icon`},e.icon?e.icon():h(xt,{clsPrefix:n},{default:this.mergedRenderIcon})):null,this.showDescription?h("div",{class:`${n}-empty__description`},e.default?e.default():this.localizedDescription):null,e.extra?h("div",{class:`${n}-empty__extra`},e.extra()):null)}}),yt=he({name:"NBaseSelectGroupHeader",props:{clsPrefix:{type:String,required:!0},tmNode:{type:Object,required:!0}},setup(){const{renderLabelRef:e,renderOptionRef:n,labelFieldRef:t,nodePropsRef:o}=Wn(Gn);return{labelField:t,nodeProps:o,renderLabel:e,renderOption:n}},render(){const{clsPrefix:e,renderLabel:n,renderOption:t,nodeProps:o,tmNode:{rawNode:r}}=this,a=o==null?void 0:o(r),i=n?n(r,!1):Le(r[this.labelField],r,!1),l=h("div",Object.assign({},a,{class:[`${e}-base-select-group-header`,a==null?void 0:a.class]}),i);return r.render?r.render({node:l,option:r}):t?t({node:l,option:r,selected:!1}):l}});function ll(e,n){return h(jn,{name:"fade-in-scale-up-transition"},{default:()=>e?h(xt,{clsPrefix:n,class:`${n}-base-select-option__check`},{default:()=>h(Mi)}):null})}const wt=he({name:"NBaseSelectOption",props:{clsPrefix:{type:String,required:!0},tmNode:{type:Object,required:!0}},setup(e){const{valueRef:n,pendingTmNodeRef:t,multipleRef:o,valueSetRef:r,renderLabelRef:a,renderOptionRef:i,labelFieldRef:l,valueFieldRef:u,showCheckmarkRef:f,nodePropsRef:d,handleOptionClick:c,handleOptionMouseEnter:C}=Wn(Gn),O=Ze(()=>{const{value:x}=t;return x?e.tmNode.key===x.key:!1});function v(x){const{tmNode:S}=e;S.disabled||c(x,S)}function g(x){const{tmNode:S}=e;S.disabled||C(x,S)}function k(x){const{tmNode:S}=e,{value:M}=O;S.disabled||M||C(x,S)}return{multiple:o,isGrouped:Ze(()=>{const{tmNode:x}=e,{parent:S}=x;return S&&S.rawNode.type==="group"}),showCheckmark:f,nodeProps:d,isPending:O,isSelected:Ze(()=>{const{value:x}=n,{value:S}=o;if(x===null)return!1;const M=e.tmNode.rawNode[u.value];if(S){const{value:m}=r;return m.has(M)}else return x===M}),labelField:l,renderLabel:a,renderOption:i,handleMouseMove:k,handleMouseEnter:g,handleClick:v}},render(){const{clsPrefix:e,tmNode:{rawNode:n},isSelected:t,isPending:o,isGrouped:r,showCheckmark:a,nodeProps:i,renderOption:l,renderLabel:u,handleClick:f,handleMouseEnter:d,handleMouseMove:c}=this,C=ll(t,e),O=u?[u(n,t),a&&C]:[Le(n[this.labelField],n,t),a&&C],v=i==null?void 0:i(n),g=h("div",Object.assign({},v,{class:[`${e}-base-select-option`,n.class,v==null?void 0:v.class,{[`${e}-base-select-option--disabled`]:n.disabled,[`${e}-base-select-option--selected`]:t,[`${e}-base-select-option--grouped`]:r,[`${e}-base-select-option--pending`]:o,[`${e}-base-select-option--show-checkmark`]:a}],style:[(v==null?void 0:v.style)||"",n.style||""],onClick:On([f,v==null?void 0:v.onClick]),onMouseenter:On([d,v==null?void 0:v.onMouseenter]),onMousemove:On([c,v==null?void 0:v.onMousemove])}),h("div",{class:`${e}-base-select-option__content`},O));return n.render?n.render({node:g,option:n,selected:t}):l?l({node:g,option:n,selected:t}):g}}),al=A("base-select-menu",`
 line-height: 1.5;
 outline: none;
 z-index: 0;
 position: relative;
 border-radius: var(--n-border-radius);
 transition:
 background-color .3s var(--n-bezier),
 box-shadow .3s var(--n-bezier);
 background-color: var(--n-color);
`,[A("scrollbar",`
 max-height: var(--n-height);
 `),A("virtual-list",`
 max-height: var(--n-height);
 `),A("base-select-option",`
 min-height: var(--n-option-height);
 font-size: var(--n-option-font-size);
 display: flex;
 align-items: center;
 `,[D("content",`
 z-index: 1;
 white-space: nowrap;
 text-overflow: ellipsis;
 overflow: hidden;
 `)]),A("base-select-group-header",`
 min-height: var(--n-option-height);
 font-size: .93em;
 display: flex;
 align-items: center;
 `),A("base-select-menu-option-wrapper",`
 position: relative;
 width: 100%;
 `),D("loading, empty",`
 display: flex;
 padding: 12px 32px;
 flex: 1;
 justify-content: center;
 `),D("loading",`
 color: var(--n-loading-color);
 font-size: var(--n-loading-size);
 `),D("header",`
 padding: 8px var(--n-option-padding-left);
 font-size: var(--n-option-font-size);
 transition: 
 color .3s var(--n-bezier),
 border-color .3s var(--n-bezier);
 border-bottom: 1px solid var(--n-action-divider-color);
 color: var(--n-action-text-color);
 `),D("action",`
 padding: 8px var(--n-option-padding-left);
 font-size: var(--n-option-font-size);
 transition: 
 color .3s var(--n-bezier),
 border-color .3s var(--n-bezier);
 border-top: 1px solid var(--n-action-divider-color);
 color: var(--n-action-text-color);
 `),A("base-select-group-header",`
 position: relative;
 cursor: default;
 padding: var(--n-option-padding);
 color: var(--n-group-header-text-color);
 `),A("base-select-option",`
 cursor: pointer;
 position: relative;
 padding: var(--n-option-padding);
 transition:
 color .3s var(--n-bezier),
 opacity .3s var(--n-bezier);
 box-sizing: border-box;
 color: var(--n-option-text-color);
 opacity: 1;
 `,[X("show-checkmark",`
 padding-right: calc(var(--n-option-padding-right) + 20px);
 `),Q("&::before",`
 content: "";
 position: absolute;
 left: 4px;
 right: 4px;
 top: 0;
 bottom: 0;
 border-radius: var(--n-border-radius);
 transition: background-color .3s var(--n-bezier);
 `),Q("&:active",`
 color: var(--n-option-text-color-pressed);
 `),X("grouped",`
 padding-left: calc(var(--n-option-padding-left) * 1.5);
 `),X("pending",[Q("&::before",`
 background-color: var(--n-option-color-pending);
 `)]),X("selected",`
 color: var(--n-option-text-color-active);
 `,[Q("&::before",`
 background-color: var(--n-option-color-active);
 `),X("pending",[Q("&::before",`
 background-color: var(--n-option-color-active-pending);
 `)])]),X("disabled",`
 cursor: not-allowed;
 `,[xe("selected",`
 color: var(--n-option-text-color-disabled);
 `),X("selected",`
 opacity: var(--n-option-opacity-disabled);
 `)]),D("check",`
 font-size: 16px;
 position: absolute;
 right: calc(var(--n-option-padding-right) - 4px);
 top: calc(50% - 7px);
 color: var(--n-option-check-color);
 transition: color .3s var(--n-bezier);
 `,[St({enterScale:"0.5"})])])]),sl=he({name:"InternalSelectMenu",props:Object.assign(Object.assign({},ce.props),{clsPrefix:{type:String,required:!0},scrollable:{type:Boolean,default:!0},treeMate:{type:Object,required:!0},multiple:Boolean,size:{type:String,default:"medium"},value:{type:[String,Number,Array],default:null},autoPending:Boolean,virtualScroll:{type:Boolean,default:!0},show:{type:Boolean,default:!0},labelField:{type:String,default:"label"},valueField:{type:String,default:"value"},loading:Boolean,focusable:Boolean,renderLabel:Function,renderOption:Function,nodeProps:Function,showCheckmark:{type:Boolean,default:!0},onMousedown:Function,onScroll:Function,onFocus:Function,onBlur:Function,onKeyup:Function,onKeydown:Function,onTabOut:Function,onMouseenter:Function,onMouseleave:Function,onResize:Function,resetMenuOnOptionsChange:{type:Boolean,default:!0},inlineThemeDisabled:Boolean,scrollbarProps:Object,onToggle:Function}),setup(e){const{mergedClsPrefixRef:n,mergedRtlRef:t,mergedComponentPropsRef:o}=Ae(e),r=hn("InternalSelectMenu",t,n),a=ce("InternalSelectMenu","-internal-select-menu",al,Io,e,te(e,"clsPrefix")),i=E(null),l=E(null),u=E(null),f=N(()=>e.treeMate.getFlattenedNodes()),d=N(()=>Wi(f.value)),c=E(null);function C(){const{treeMate:b}=e;let P=null;const{value:J}=e;J===null?P=b.getFirstAvailableNode():(e.multiple?P=b.getNode((J||[])[(J||[]).length-1]):P=b.getNode(J),(!P||P.disabled)&&(P=b.getFirstAvailableNode())),G(P||null)}function O(){const{value:b}=c;b&&!e.treeMate.getNode(b.key)&&(c.value=null)}let v;_e(()=>e.show,b=>{b?v=_e(()=>e.treeMate,()=>{e.resetMenuOnOptionsChange?(e.autoPending?C():O(),Dn(q)):O()},{immediate:!0}):v==null||v()},{immediate:!0}),Hn(()=>{v==null||v()});const g=N(()=>Ao(a.value.self[ee("optionHeight",e.size)])),k=N(()=>De(a.value.self[ee("padding",e.size)])),x=N(()=>e.multiple&&Array.isArray(e.value)?new Set(e.value):new Set),S=N(()=>{const b=f.value;return b&&b.length===0}),M=N(()=>{var b,P;return(P=(b=o==null?void 0:o.value)===null||b===void 0?void 0:b.Select)===null||P===void 0?void 0:P.renderEmpty});function m(b){const{onToggle:P}=e;P&&P(b)}function y(b){const{onScroll:P}=e;P&&P(b)}function z(b){var P;(P=u.value)===null||P===void 0||P.sync(),y(b)}function K(){var b;(b=u.value)===null||b===void 0||b.sync()}function j(){const{value:b}=c;return b||null}function T(b,P){P.disabled||G(P,!1)}function F(b,P){P.disabled||m(P)}function W(b){var P;Xe(b,"action")||(P=e.onKeyup)===null||P===void 0||P.call(e,b)}function I(b){var P;Xe(b,"action")||(P=e.onKeydown)===null||P===void 0||P.call(e,b)}function U(b){var P;(P=e.onMousedown)===null||P===void 0||P.call(e,b),!e.focusable&&b.preventDefault()}function ue(){const{value:b}=c;b&&G(b.getNext({loop:!0}),!0)}function _(){const{value:b}=c;b&&G(b.getPrev({loop:!0}),!0)}function G(b,P=!1){c.value=b,P&&q()}function q(){var b,P;const J=c.value;if(!J)return;const ve=d.value(J.key);ve!==null&&(e.virtualScroll?(b=l.value)===null||b===void 0||b.scrollTo({index:ve}):(P=u.value)===null||P===void 0||P.scrollTo({index:ve,elSize:g.value}))}function ae(b){var P,J;!((P=i.value)===null||P===void 0)&&P.contains(b.target)&&((J=e.onFocus)===null||J===void 0||J.call(e,b))}function le(b){var P,J;!((P=i.value)===null||P===void 0)&&P.contains(b.relatedTarget)||(J=e.onBlur)===null||J===void 0||J.call(e,b)}Fe(Gn,{handleOptionMouseEnter:T,handleOptionClick:F,valueSetRef:x,pendingTmNodeRef:c,nodePropsRef:te(e,"nodeProps"),showCheckmarkRef:te(e,"showCheckmark"),multipleRef:te(e,"multiple"),valueRef:te(e,"value"),renderLabelRef:te(e,"renderLabel"),renderOptionRef:te(e,"renderOption"),labelFieldRef:te(e,"labelField"),valueFieldRef:te(e,"valueField")}),Fe(dr,i),cn(()=>{const{value:b}=u;b&&b.sync()});const fe=N(()=>{const{size:b}=e,{common:{cubicBezierEaseInOut:P},self:{height:J,borderRadius:ve,color:Se,groupHeaderTextColor:be,actionDividerColor:se,optionTextColorPressed:Re,optionTextColor:ye,optionTextColorDisabled:we,optionTextColorActive:je,optionOpacityDisabled:Ve,optionCheckColor:Me,actionTextColor:ze,optionColorPending:Ue,optionColorActive:Ge,loadingColor:qe,loadingSize:Be,optionColorActivePending:Ee,[ee("optionFontSize",b)]:me,[ee("optionHeight",b)]:p,[ee("optionPadding",b)]:R}}=a.value;return{"--n-height":J,"--n-action-divider-color":se,"--n-action-text-color":ze,"--n-bezier":P,"--n-border-radius":ve,"--n-color":Se,"--n-option-font-size":me,"--n-group-header-text-color":be,"--n-option-check-color":Me,"--n-option-color-pending":Ue,"--n-option-color-active":Ge,"--n-option-color-active-pending":Ee,"--n-option-height":p,"--n-option-opacity-disabled":Ve,"--n-option-text-color":ye,"--n-option-text-color-active":je,"--n-option-text-color-disabled":we,"--n-option-text-color-pressed":Re,"--n-option-padding":R,"--n-option-padding-left":De(R,"left"),"--n-option-padding-right":De(R,"right"),"--n-loading-color":qe,"--n-loading-size":Be}}),{inlineThemeDisabled:oe}=e,Y=oe?We("internal-select-menu",N(()=>e.size[0]),fe,e):void 0,pe={selfRef:i,next:ue,prev:_,getPendingTmNode:j};return Ft(i,e.onResize),Object.assign({mergedTheme:a,mergedClsPrefix:n,rtlEnabled:r,virtualListRef:l,scrollbarRef:u,itemSize:g,padding:k,flattenedNodes:f,empty:S,mergedRenderEmpty:M,virtualListContainer(){const{value:b}=l;return b==null?void 0:b.listElRef},virtualListContent(){const{value:b}=l;return b==null?void 0:b.itemsElRef},doScroll:y,handleFocusin:ae,handleFocusout:le,handleKeyUp:W,handleKeyDown:I,handleMouseDown:U,handleVirtualListResize:K,handleVirtualListScroll:z,cssVars:oe?void 0:fe,themeClass:Y==null?void 0:Y.themeClass,onRender:Y==null?void 0:Y.onRender},pe)},render(){const{$slots:e,virtualScroll:n,clsPrefix:t,mergedTheme:o,themeClass:r,onRender:a}=this;return a==null||a(),h("div",{ref:"selfRef",tabindex:this.focusable?0:-1,class:[`${t}-base-select-menu`,`${t}-base-select-menu--${this.size}-size`,this.rtlEnabled&&`${t}-base-select-menu--rtl`,r,this.multiple&&`${t}-base-select-menu--multiple`],style:this.cssVars,onFocusin:this.handleFocusin,onFocusout:this.handleFocusout,onKeyup:this.handleKeyUp,onKeydown:this.handleKeyDown,onMousedown:this.handleMouseDown,onMouseenter:this.onMouseenter,onMouseleave:this.onMouseleave},$e(e.header,i=>i&&h("div",{class:`${t}-base-select-menu__header`,"data-header":!0,key:"header"},i)),this.loading?h("div",{class:`${t}-base-select-menu__loading`},h(zo,{clsPrefix:t,strokeWidth:20})):this.empty?h("div",{class:`${t}-base-select-menu__empty`,"data-empty":!0},Fo(e.empty,()=>{var i;return[((i=this.mergedRenderEmpty)===null||i===void 0?void 0:i.call(this))||h(il,{theme:o.peers.Empty,themeOverrides:o.peerOverrides.Empty,size:this.size})]})):h($o,Object.assign({ref:"scrollbarRef",theme:o.peers.Scrollbar,themeOverrides:o.peerOverrides.Scrollbar,scrollable:this.scrollable,container:n?this.virtualListContainer:void 0,content:n?this.virtualListContent:void 0,onScroll:n?void 0:this.doScroll},this.scrollbarProps),{default:()=>n?h(ar,{ref:"virtualListRef",class:`${t}-virtual-list`,items:this.flattenedNodes,itemSize:this.itemSize,showScrollbar:!1,paddingTop:this.padding.top,paddingBottom:this.padding.bottom,onResize:this.handleVirtualListResize,onScroll:this.handleVirtualListScroll,itemResizable:!0},{default:({item:i})=>i.isGroup?h(yt,{key:i.key,clsPrefix:t,tmNode:i}):i.ignored?null:h(wt,{clsPrefix:t,key:i.key,tmNode:i})}):h("div",{class:`${t}-base-select-menu-option-wrapper`,style:{paddingTop:this.padding.top,paddingBottom:this.padding.bottom}},this.flattenedNodes.map(i=>i.isGroup?h(yt,{key:i.key,clsPrefix:t,tmNode:i}):h(wt,{clsPrefix:t,key:i.key,tmNode:i})))}),$e(e.action,i=>i&&[h("div",{class:`${t}-base-select-menu__action`,"data-action":!0,key:"action"},i),h(sr,{onFocus:this.onTabOut,key:"focus-detector"})]))}}),_n={top:"bottom",bottom:"top",left:"right",right:"left"},ie="var(--n-arrow-height) * 1.414",dl=Q([A("popover",`
 transition:
 box-shadow .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 color .3s var(--n-bezier);
 position: relative;
 font-size: var(--n-font-size);
 color: var(--n-text-color);
 box-shadow: var(--n-box-shadow);
 word-break: break-word;
 `,[Q(">",[A("scrollbar",`
 height: inherit;
 max-height: inherit;
 `)]),xe("raw",`
 background-color: var(--n-color);
 border-radius: var(--n-border-radius);
 `,[xe("scrollable",[xe("show-header-or-footer","padding: var(--n-padding);")])]),D("header",`
 padding: var(--n-padding);
 border-bottom: 1px solid var(--n-divider-color);
 transition: border-color .3s var(--n-bezier);
 `),D("footer",`
 padding: var(--n-padding);
 border-top: 1px solid var(--n-divider-color);
 transition: border-color .3s var(--n-bezier);
 `),X("scrollable, show-header-or-footer",[D("content",`
 padding: var(--n-padding);
 `)])]),A("popover-shared",`
 transform-origin: inherit;
 `,[A("popover-arrow-wrapper",`
 position: absolute;
 overflow: hidden;
 pointer-events: none;
 `,[A("popover-arrow",`
 transition: background-color .3s var(--n-bezier);
 position: absolute;
 display: block;
 width: calc(${ie});
 height: calc(${ie});
 box-shadow: 0 0 8px 0 rgba(0, 0, 0, .12);
 transform: rotate(45deg);
 background-color: var(--n-color);
 pointer-events: all;
 `)]),Q("&.popover-transition-enter-from, &.popover-transition-leave-to",`
 opacity: 0;
 transform: scale(.85);
 `),Q("&.popover-transition-enter-to, &.popover-transition-leave-from",`
 transform: scale(1);
 opacity: 1;
 `),Q("&.popover-transition-enter-active",`
 transition:
 box-shadow .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 color .3s var(--n-bezier),
 opacity .15s var(--n-bezier-ease-out),
 transform .15s var(--n-bezier-ease-out);
 `),Q("&.popover-transition-leave-active",`
 transition:
 box-shadow .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 color .3s var(--n-bezier),
 opacity .15s var(--n-bezier-ease-in),
 transform .15s var(--n-bezier-ease-in);
 `)]),ge("top-start",`
 top: calc(${ie} / -2);
 left: calc(${Pe("top-start")} - var(--v-offset-left));
 `),ge("top",`
 top: calc(${ie} / -2);
 transform: translateX(calc(${ie} / -2)) rotate(45deg);
 left: 50%;
 `),ge("top-end",`
 top: calc(${ie} / -2);
 right: calc(${Pe("top-end")} + var(--v-offset-left));
 `),ge("bottom-start",`
 bottom: calc(${ie} / -2);
 left: calc(${Pe("bottom-start")} - var(--v-offset-left));
 `),ge("bottom",`
 bottom: calc(${ie} / -2);
 transform: translateX(calc(${ie} / -2)) rotate(45deg);
 left: 50%;
 `),ge("bottom-end",`
 bottom: calc(${ie} / -2);
 right: calc(${Pe("bottom-end")} + var(--v-offset-left));
 `),ge("left-start",`
 left: calc(${ie} / -2);
 top: calc(${Pe("left-start")} - var(--v-offset-top));
 `),ge("left",`
 left: calc(${ie} / -2);
 transform: translateY(calc(${ie} / -2)) rotate(45deg);
 top: 50%;
 `),ge("left-end",`
 left: calc(${ie} / -2);
 bottom: calc(${Pe("left-end")} + var(--v-offset-top));
 `),ge("right-start",`
 right: calc(${ie} / -2);
 top: calc(${Pe("right-start")} - var(--v-offset-top));
 `),ge("right",`
 right: calc(${ie} / -2);
 transform: translateY(calc(${ie} / -2)) rotate(45deg);
 top: 50%;
 `),ge("right-end",`
 right: calc(${ie} / -2);
 bottom: calc(${Pe("right-end")} + var(--v-offset-top));
 `),..._i({top:["right-start","left-start"],right:["top-end","bottom-end"],bottom:["right-end","left-end"],left:["top-start","bottom-start"]},(e,n)=>{const t=["right","left"].includes(n),o=t?"width":"height";return e.map(r=>{const a=r.split("-")[1]==="end",l=`calc((${`var(--v-target-${o}, 0px)`} - ${ie}) / 2)`,u=Pe(r);return Q(`[v-placement="${r}"] >`,[A("popover-shared",[X("center-arrow",[A("popover-arrow",`${n}: calc(max(${l}, ${u}) ${a?"+":"-"} var(--v-offset-${t?"left":"top"}));`)])])])})})]);function Pe(e){return["top","bottom"].includes(e.split("-")[0])?"var(--n-arrow-offset)":"var(--n-arrow-offset-vertical)"}function ge(e,n){const t=e.split("-")[0],o=["top","bottom"].includes(t)?"height: var(--n-space-arrow);":"width: var(--n-space-arrow);";return Q(`[v-placement="${e}"] >`,[A("popover-shared",`
 margin-${_n[t]}: var(--n-space);
 `,[X("show-arrow",`
 margin-${_n[t]}: var(--n-space-arrow);
 `),X("overlap",`
 margin: 0;
 `),Bo("popover-arrow-wrapper",`
 right: 0;
 left: 0;
 top: 0;
 bottom: 0;
 ${t}: 100%;
 ${_n[t]}: auto;
 ${o}
 `,[A("popover-arrow",n)])])])}const Lt=Object.assign(Object.assign({},ce.props),{to:Ie.propTo,show:Boolean,trigger:String,showArrow:Boolean,delay:Number,duration:Number,raw:Boolean,arrowPointToCenter:Boolean,arrowClass:String,arrowStyle:[String,Object],arrowWrapperClass:String,arrowWrapperStyle:[String,Object],displayDirective:String,x:Number,y:Number,flip:Boolean,overlap:Boolean,placement:String,width:[Number,String],keepAliveOnHover:Boolean,scrollable:Boolean,contentClass:String,contentStyle:[Object,String],headerClass:String,headerStyle:[Object,String],footerClass:String,footerStyle:[Object,String],internalDeactivateImmediately:Boolean,animated:Boolean,onClickoutside:Function,internalTrapFocus:Boolean,internalOnAfterLeave:Function,minWidth:Number,maxWidth:Number});function cl({arrowClass:e,arrowStyle:n,arrowWrapperClass:t,arrowWrapperStyle:o,clsPrefix:r}){return h("div",{key:"__popover-arrow__",style:o,class:[`${r}-popover-arrow-wrapper`,t]},h("div",{class:[`${r}-popover-arrow`,e],style:n}))}const ul=he({name:"PopoverBody",inheritAttrs:!1,props:Lt,setup(e,{slots:n,attrs:t}){const{namespaceRef:o,mergedClsPrefixRef:r,inlineThemeDisabled:a,mergedRtlRef:i}=Ae(e),l=ce("Popover","-popover",dl,Do,e,r),u=hn("Popover",i,r),f=E(null),d=Wn("NPopover"),c=E(null),C=E(e.show),O=E(!1);Vn(()=>{const{show:T}=e;T&&!pr()&&!e.internalDeactivateImmediately&&(O.value=!0)});const v=N(()=>{const{trigger:T,onClickoutside:F}=e,W=[],{positionManuallyRef:{value:I}}=d;return I||(T==="click"&&!F&&W.push([ln,z,void 0,{capture:!0}]),T==="hover"&&W.push([vr,y])),F&&W.push([ln,z,void 0,{capture:!0}]),(e.displayDirective==="show"||e.animated&&O.value)&&W.push([Pt,e.show]),W}),g=N(()=>{const{common:{cubicBezierEaseInOut:T,cubicBezierEaseIn:F,cubicBezierEaseOut:W},self:{space:I,spaceArrow:U,padding:ue,fontSize:_,textColor:G,dividerColor:q,color:ae,boxShadow:le,borderRadius:fe,arrowHeight:oe,arrowOffset:Y,arrowOffsetVertical:pe}}=l.value;return{"--n-box-shadow":le,"--n-bezier":T,"--n-bezier-ease-in":F,"--n-bezier-ease-out":W,"--n-font-size":_,"--n-text-color":G,"--n-color":ae,"--n-divider-color":q,"--n-border-radius":fe,"--n-arrow-height":oe,"--n-arrow-offset":Y,"--n-arrow-offset-vertical":pe,"--n-padding":ue,"--n-space":I,"--n-space-arrow":U}}),k=N(()=>{const T=e.width==="trigger"?void 0:xn(e.width),F=[];T&&F.push({width:T});const{maxWidth:W,minWidth:I}=e;return W&&F.push({maxWidth:xn(W)}),I&&F.push({maxWidth:xn(I)}),a||F.push(g.value),F}),x=a?We("popover",void 0,g,e):void 0;d.setBodyInstance({syncPosition:S}),Hn(()=>{d.setBodyInstance(null)}),_e(te(e,"show"),T=>{e.animated||(T?C.value=!0:C.value=!1)});function S(){var T;(T=f.value)===null||T===void 0||T.syncPosition()}function M(T){e.trigger==="hover"&&e.keepAliveOnHover&&e.show&&d.handleMouseEnter(T)}function m(T){e.trigger==="hover"&&e.keepAliveOnHover&&d.handleMouseLeave(T)}function y(T){e.trigger==="hover"&&!K().contains(Fn(T))&&d.handleMouseMoveOutside(T)}function z(T){(e.trigger==="click"&&!K().contains(Fn(T))||e.onClickoutside)&&d.handleClickOutside(T)}function K(){return d.getTriggerElement()}Fe(Ho,c),Fe(Ko,null),Fe(Wo,null);function j(){if(x==null||x.onRender(),!(e.displayDirective==="show"||e.show||e.animated&&O.value))return null;let F;const W=d.internalRenderBodyRef.value,{value:I}=r;if(W)F=W([`${I}-popover-shared`,(u==null?void 0:u.value)&&`${I}-popover--rtl`,x==null?void 0:x.themeClass.value,e.overlap&&`${I}-popover-shared--overlap`,e.showArrow&&`${I}-popover-shared--show-arrow`,e.arrowPointToCenter&&`${I}-popover-shared--center-arrow`],c,k.value,M,m);else{const{value:U}=d.extraClassRef,{internalTrapFocus:ue}=e,_=!tt(n.header)||!tt(n.footer),G=()=>{var q,ae;const le=_?h(Ot,null,$e(n.header,Y=>Y?h("div",{class:[`${I}-popover__header`,e.headerClass],style:e.headerStyle},Y):null),$e(n.default,Y=>Y?h("div",{class:[`${I}-popover__content`,e.contentClass],style:e.contentStyle},n):null),$e(n.footer,Y=>Y?h("div",{class:[`${I}-popover__footer`,e.footerClass],style:e.footerStyle},Y):null)):e.scrollable?(q=n.default)===null||q===void 0?void 0:q.call(n):h("div",{class:[`${I}-popover__content`,e.contentClass],style:e.contentStyle},n),fe=e.scrollable?h(Lo,{themeOverrides:l.value.peerOverrides.Scrollbar,theme:l.value.peers.Scrollbar,contentClass:_?void 0:`${I}-popover__content ${(ae=e.contentClass)!==null&&ae!==void 0?ae:""}`,contentStyle:_?void 0:e.contentStyle},{default:()=>le}):le,oe=e.showArrow?cl({arrowClass:e.arrowClass,arrowStyle:e.arrowStyle,arrowWrapperClass:e.arrowWrapperClass,arrowWrapperStyle:e.arrowWrapperStyle,clsPrefix:I}):null;return[fe,oe]};F=h("div",No({class:[`${I}-popover`,`${I}-popover-shared`,(u==null?void 0:u.value)&&`${I}-popover--rtl`,x==null?void 0:x.themeClass.value,U.map(q=>`${I}-${q}`),{[`${I}-popover--scrollable`]:e.scrollable,[`${I}-popover--show-header-or-footer`]:_,[`${I}-popover--raw`]:e.raw,[`${I}-popover-shared--overlap`]:e.overlap,[`${I}-popover-shared--show-arrow`]:e.showArrow,[`${I}-popover-shared--center-arrow`]:e.arrowPointToCenter}],ref:c,style:k.value,onKeydown:d.handleKeydown,onMouseenter:M,onMouseleave:m},t),ue?h(Eo,{active:e.show,autoFocus:!0},{default:G}):G())}return Un(F,v.value)}return{displayed:O,namespace:o,isMounted:d.isMountedRef,zIndex:d.zIndexRef,followerRef:f,adjustedTo:Ie(e),followerEnabled:C,renderContentNode:j}},render(){return h(kt,{ref:"followerRef",zIndex:this.zIndex,show:this.show,enabled:this.followerEnabled,to:this.adjustedTo,x:this.x,y:this.y,flip:this.flip,placement:this.placement,containerClass:this.namespace,overlap:this.overlap,width:this.width==="trigger"?"target":void 0,teleportDisabled:this.adjustedTo===Ie.tdkey},{default:()=>this.animated?h(jn,{name:"popover-transition",appear:this.isMounted,onEnter:()=>{this.followerEnabled=!0},onAfterLeave:()=>{var e;(e=this.internalOnAfterLeave)===null||e===void 0||e.call(this),this.followerEnabled=!1,this.displayed=!1}},{default:this.renderContentNode}):this.renderContentNode()})}}),fl=Object.keys(Lt),hl={focus:["onFocus","onBlur"],click:["onClick"],hover:["onMouseenter","onMouseleave"],manual:[],nested:["onFocus","onBlur","onMouseenter","onMouseleave","onClick"]};function vl(e,n,t){hl[n].forEach(o=>{e.props?e.props=Object.assign({},e.props):e.props={};const r=e.props[o],a=t[o];r?e.props[o]=(...i)=>{r(...i),a(...i)}:e.props[o]=a})}const Dt={show:{type:Boolean,default:void 0},defaultShow:Boolean,showArrow:{type:Boolean,default:!0},trigger:{type:String,default:"hover"},delay:{type:Number,default:100},duration:{type:Number,default:100},raw:Boolean,placement:{type:String,default:"top"},x:Number,y:Number,arrowPointToCenter:Boolean,disabled:Boolean,getDisabled:Function,displayDirective:{type:String,default:"if"},arrowClass:String,arrowStyle:[String,Object],arrowWrapperClass:String,arrowWrapperStyle:[String,Object],flip:{type:Boolean,default:!0},animated:{type:Boolean,default:!0},width:{type:[Number,String],default:void 0},overlap:Boolean,keepAliveOnHover:{type:Boolean,default:!0},zIndex:Number,to:Ie.propTo,scrollable:Boolean,contentClass:String,contentStyle:[Object,String],headerClass:String,headerStyle:[Object,String],footerClass:String,footerStyle:[Object,String],onClickoutside:Function,"onUpdate:show":[Function,Array],onUpdateShow:[Function,Array],internalDeactivateImmediately:Boolean,internalSyncTargetWithParent:Boolean,internalInheritedEventHandlers:{type:Array,default:()=>[]},internalTrapFocus:Boolean,internalExtraClass:{type:Array,default:()=>[]},onShow:[Function,Array],onHide:[Function,Array],arrow:{type:Boolean,default:void 0},minWidth:Number,maxWidth:Number},gl=Object.assign(Object.assign(Object.assign({},ce.props),Dt),{internalOnAfterLeave:Function,internalRenderBody:Function}),Ht=he({name:"Popover",inheritAttrs:!1,props:gl,slots:Object,__popover__:!0,setup(e){const n=Rt(),t=E(null),o=N(()=>e.show),r=E(e.defaultShow),a=In(o,r),i=Ze(()=>e.disabled?!1:a.value),l=()=>{if(e.disabled)return!0;const{getDisabled:_}=e;return!!(_!=null&&_())},u=()=>l()?!1:a.value,f=$t(e,["arrow","showArrow"]),d=N(()=>e.overlap?!1:f.value);let c=null;const C=E(null),O=E(null),v=Ze(()=>e.x!==void 0&&e.y!==void 0);function g(_){const{"onUpdate:show":G,onUpdateShow:q,onShow:ae,onHide:le}=e;r.value=_,G&&de(G,_),q&&de(q,_),_&&ae&&de(ae,!0),_&&le&&de(le,!1)}function k(){c&&c.syncPosition()}function x(){const{value:_}=C;_&&(window.clearTimeout(_),C.value=null)}function S(){const{value:_}=O;_&&(window.clearTimeout(_),O.value=null)}function M(){const _=l();if(e.trigger==="focus"&&!_){if(u())return;g(!0)}}function m(){const _=l();if(e.trigger==="focus"&&!_){if(!u())return;g(!1)}}function y(){const _=l();if(e.trigger==="hover"&&!_){if(S(),C.value!==null||u())return;const G=()=>{g(!0),C.value=null},{delay:q}=e;q===0?G():C.value=window.setTimeout(G,q)}}function z(){const _=l();if(e.trigger==="hover"&&!_){if(x(),O.value!==null||!u())return;const G=()=>{g(!1),O.value=null},{duration:q}=e;q===0?G():O.value=window.setTimeout(G,q)}}function K(){z()}function j(_){var G;u()&&(e.trigger==="click"&&(x(),S(),g(!1)),(G=e.onClickoutside)===null||G===void 0||G.call(e,_))}function T(){if(e.trigger==="click"&&!l()){x(),S();const _=!u();g(_)}}function F(_){e.internalTrapFocus&&_.key==="Escape"&&(x(),S(),g(!1))}function W(_){r.value=_}function I(){var _;return(_=t.value)===null||_===void 0?void 0:_.targetRef}function U(_){c=_}return Fe("NPopover",{getTriggerElement:I,handleKeydown:F,handleMouseEnter:y,handleMouseLeave:z,handleClickOutside:j,handleMouseMoveOutside:K,setBodyInstance:U,positionManuallyRef:v,isMountedRef:n,zIndexRef:te(e,"zIndex"),extraClassRef:te(e,"internalExtraClass"),internalRenderBodyRef:te(e,"internalRenderBody")}),Vn(()=>{a.value&&l()&&g(!1)}),{binderInstRef:t,positionManually:v,mergedShowConsideringDisabledProp:i,uncontrolledShow:r,mergedShowArrow:d,getMergedShow:u,setShow:W,handleClick:T,handleMouseEnter:y,handleMouseLeave:z,handleFocus:M,handleBlur:m,syncPosition:k}},render(){var e;const{positionManually:n,$slots:t}=this;let o,r=!1;if(!n&&(o=jo(t,"trigger"),o)){o=Vo(o),o=o.type===Uo?h("span",[o]):o;const a={onClick:this.handleClick,onMouseenter:this.handleMouseEnter,onMouseleave:this.handleMouseLeave,onFocus:this.handleFocus,onBlur:this.handleBlur};if(!((e=o.type)===null||e===void 0)&&e.__popover__)r=!0,o.props||(o.props={internalSyncTargetWithParent:!0,internalInheritedEventHandlers:[]}),o.props.internalSyncTargetWithParent=!0,o.props.internalInheritedEventHandlers?o.props.internalInheritedEventHandlers=[a,...o.props.internalInheritedEventHandlers]:o.props.internalInheritedEventHandlers=[a];else{const{internalInheritedEventHandlers:i}=this,l=[a,...i],u={onBlur:f=>{l.forEach(d=>{d.onBlur(f)})},onFocus:f=>{l.forEach(d=>{d.onFocus(f)})},onClick:f=>{l.forEach(d=>{d.onClick(f)})},onMouseenter:f=>{l.forEach(d=>{d.onMouseenter(f)})},onMouseleave:f=>{l.forEach(d=>{d.onMouseleave(f)})}};vl(o,i?"nested":n?"manual":this.trigger,u)}}return h(Tt,{ref:"binderInstRef",syncTarget:!r,syncTargetWithParent:this.internalSyncTargetWithParent},{default:()=>{this.mergedShowConsideringDisabledProp;const a=this.getMergedShow();return[this.internalTrapFocus&&a?Un(h("div",{style:{position:"fixed",top:0,right:0,bottom:0,left:0}}),[[Go,{enabled:a,zIndex:this.zIndex}]]):null,n?null:h(_t,null,{default:()=>o}),h(ul,qo(this.$props,fl,Object.assign(Object.assign({},this.$attrs),{showArrow:this.mergedShowArrow,show:a})),{default:()=>{var i,l;return(l=(i=this.$slots).default)===null||l===void 0?void 0:l.call(i)},header:()=>{var i,l;return(l=(i=this.$slots).header)===null||l===void 0?void 0:l.call(i)},footer:()=>{var i,l;return(l=(i=this.$slots).footer)===null||l===void 0?void 0:l.call(i)}})]}})}});function pl(e){const{textColor2:n,primaryColorHover:t,primaryColorPressed:o,primaryColor:r,infoColor:a,successColor:i,warningColor:l,errorColor:u,baseColor:f,borderColor:d,opacityDisabled:c,tagColor:C,closeIconColor:O,closeIconColorHover:v,closeIconColorPressed:g,borderRadiusSmall:k,fontSizeMini:x,fontSizeTiny:S,fontSizeSmall:M,fontSizeMedium:m,heightMini:y,heightTiny:z,heightSmall:K,heightMedium:j,closeColorHover:T,closeColorPressed:F,buttonColor2Hover:W,buttonColor2Pressed:I,fontWeightStrong:U}=e;return Object.assign(Object.assign({},Xo),{closeBorderRadius:k,heightTiny:y,heightSmall:z,heightMedium:K,heightLarge:j,borderRadius:k,opacityDisabled:c,fontSizeTiny:x,fontSizeSmall:S,fontSizeMedium:M,fontSizeLarge:m,fontWeightStrong:U,textColorCheckable:n,textColorHoverCheckable:n,textColorPressedCheckable:n,textColorChecked:f,colorCheckable:"#0000",colorHoverCheckable:W,colorPressedCheckable:I,colorChecked:r,colorCheckedHover:t,colorCheckedPressed:o,border:`1px solid ${d}`,textColor:n,color:C,colorBordered:"rgb(250, 250, 252)",closeIconColor:O,closeIconColorHover:v,closeIconColorPressed:g,closeColorHover:T,closeColorPressed:F,borderPrimary:`1px solid ${Z(r,{alpha:.3})}`,textColorPrimary:r,colorPrimary:Z(r,{alpha:.12}),colorBorderedPrimary:Z(r,{alpha:.1}),closeIconColorPrimary:r,closeIconColorHoverPrimary:r,closeIconColorPressedPrimary:r,closeColorHoverPrimary:Z(r,{alpha:.12}),closeColorPressedPrimary:Z(r,{alpha:.18}),borderInfo:`1px solid ${Z(a,{alpha:.3})}`,textColorInfo:a,colorInfo:Z(a,{alpha:.12}),colorBorderedInfo:Z(a,{alpha:.1}),closeIconColorInfo:a,closeIconColorHoverInfo:a,closeIconColorPressedInfo:a,closeColorHoverInfo:Z(a,{alpha:.12}),closeColorPressedInfo:Z(a,{alpha:.18}),borderSuccess:`1px solid ${Z(i,{alpha:.3})}`,textColorSuccess:i,colorSuccess:Z(i,{alpha:.12}),colorBorderedSuccess:Z(i,{alpha:.1}),closeIconColorSuccess:i,closeIconColorHoverSuccess:i,closeIconColorPressedSuccess:i,closeColorHoverSuccess:Z(i,{alpha:.12}),closeColorPressedSuccess:Z(i,{alpha:.18}),borderWarning:`1px solid ${Z(l,{alpha:.35})}`,textColorWarning:l,colorWarning:Z(l,{alpha:.15}),colorBorderedWarning:Z(l,{alpha:.12}),closeIconColorWarning:l,closeIconColorHoverWarning:l,closeIconColorPressedWarning:l,closeColorHoverWarning:Z(l,{alpha:.12}),closeColorPressedWarning:Z(l,{alpha:.18}),borderError:`1px solid ${Z(u,{alpha:.23})}`,textColorError:u,colorError:Z(u,{alpha:.1}),colorBorderedError:Z(u,{alpha:.08}),closeIconColorError:u,closeIconColorHoverError:u,closeIconColorPressedError:u,closeColorHoverError:Z(u,{alpha:.12}),closeColorPressedError:Z(u,{alpha:.18})})}const bl={name:"Tag",common:Zo,self:pl},ml={color:Object,type:{type:String,default:"default"},round:Boolean,size:String,closable:Boolean,disabled:{type:Boolean,default:void 0}},yl=A("tag",`
 --n-close-margin: var(--n-close-margin-top) var(--n-close-margin-right) var(--n-close-margin-bottom) var(--n-close-margin-left);
 white-space: nowrap;
 position: relative;
 box-sizing: border-box;
 cursor: default;
 display: inline-flex;
 align-items: center;
 flex-wrap: nowrap;
 padding: var(--n-padding);
 border-radius: var(--n-border-radius);
 color: var(--n-text-color);
 background-color: var(--n-color);
 transition: 
 border-color .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 color .3s var(--n-bezier),
 box-shadow .3s var(--n-bezier),
 opacity .3s var(--n-bezier);
 line-height: 1;
 height: var(--n-height);
 font-size: var(--n-font-size);
`,[X("strong",`
 font-weight: var(--n-font-weight-strong);
 `),D("border",`
 pointer-events: none;
 position: absolute;
 left: 0;
 right: 0;
 top: 0;
 bottom: 0;
 border-radius: inherit;
 border: var(--n-border);
 transition: border-color .3s var(--n-bezier);
 `),D("icon",`
 display: flex;
 margin: 0 4px 0 0;
 color: var(--n-text-color);
 transition: color .3s var(--n-bezier);
 font-size: var(--n-avatar-size-override);
 `),D("avatar",`
 display: flex;
 margin: 0 6px 0 0;
 `),D("close",`
 margin: var(--n-close-margin);
 transition:
 background-color .3s var(--n-bezier),
 color .3s var(--n-bezier);
 `),X("round",`
 padding: 0 calc(var(--n-height) / 3);
 border-radius: calc(var(--n-height) / 2);
 `,[D("icon",`
 margin: 0 4px 0 calc((var(--n-height) - 8px) / -2);
 `),D("avatar",`
 margin: 0 6px 0 calc((var(--n-height) - 8px) / -2);
 `),X("closable",`
 padding: 0 calc(var(--n-height) / 4) 0 calc(var(--n-height) / 3);
 `)]),X("icon, avatar",[X("round",`
 padding: 0 calc(var(--n-height) / 3) 0 calc(var(--n-height) / 2);
 `)]),X("disabled",`
 cursor: not-allowed !important;
 opacity: var(--n-opacity-disabled);
 `),X("checkable",`
 cursor: pointer;
 box-shadow: none;
 color: var(--n-text-color-checkable);
 background-color: var(--n-color-checkable);
 `,[xe("disabled",[Q("&:hover","background-color: var(--n-color-hover-checkable);",[xe("checked","color: var(--n-text-color-hover-checkable);")]),Q("&:active","background-color: var(--n-color-pressed-checkable);",[xe("checked","color: var(--n-text-color-pressed-checkable);")])]),X("checked",`
 color: var(--n-text-color-checked);
 background-color: var(--n-color-checked);
 `,[xe("disabled",[Q("&:hover","background-color: var(--n-color-checked-hover);"),Q("&:active","background-color: var(--n-color-checked-pressed);")])])])]),wl=Object.assign(Object.assign(Object.assign({},ce.props),ml),{bordered:{type:Boolean,default:void 0},checked:Boolean,checkable:Boolean,strong:Boolean,triggerClickOnClose:Boolean,onClose:[Array,Function],onMouseenter:Function,onMouseleave:Function,"onUpdate:checked":Function,onUpdateChecked:Function,internalCloseFocusable:{type:Boolean,default:!0},internalCloseIsButtonTag:{type:Boolean,default:!0},onCheckedChange:Function}),Cl=Jo("n-tag"),Mn=he({name:"Tag",props:wl,slots:Object,setup(e){const n=E(null),{mergedBorderedRef:t,mergedClsPrefixRef:o,inlineThemeDisabled:r,mergedRtlRef:a,mergedComponentPropsRef:i}=Ae(e),l=N(()=>{var g,k;return e.size||((k=(g=i==null?void 0:i.value)===null||g===void 0?void 0:g.Tag)===null||k===void 0?void 0:k.size)||"medium"}),u=ce("Tag","-tag",yl,bl,e,o);Fe(Cl,{roundRef:te(e,"round")});function f(){if(!e.disabled&&e.checkable){const{checked:g,onCheckedChange:k,onUpdateChecked:x,"onUpdate:checked":S}=e;x&&x(!g),S&&S(!g),k&&k(!g)}}function d(g){if(e.triggerClickOnClose||g.stopPropagation(),!e.disabled){const{onClose:k}=e;k&&de(k,g)}}const c={setTextContent(g){const{value:k}=n;k&&(k.textContent=g)}},C=hn("Tag",a,o),O=N(()=>{const{type:g,color:{color:k,textColor:x}={}}=e,S=l.value,{common:{cubicBezierEaseInOut:M},self:{padding:m,closeMargin:y,borderRadius:z,opacityDisabled:K,textColorCheckable:j,textColorHoverCheckable:T,textColorPressedCheckable:F,textColorChecked:W,colorCheckable:I,colorHoverCheckable:U,colorPressedCheckable:ue,colorChecked:_,colorCheckedHover:G,colorCheckedPressed:q,closeBorderRadius:ae,fontWeightStrong:le,[ee("colorBordered",g)]:fe,[ee("closeSize",S)]:oe,[ee("closeIconSize",S)]:Y,[ee("fontSize",S)]:pe,[ee("height",S)]:b,[ee("color",g)]:P,[ee("textColor",g)]:J,[ee("border",g)]:ve,[ee("closeIconColor",g)]:Se,[ee("closeIconColorHover",g)]:be,[ee("closeIconColorPressed",g)]:se,[ee("closeColorHover",g)]:Re,[ee("closeColorPressed",g)]:ye}}=u.value,we=De(y);return{"--n-font-weight-strong":le,"--n-avatar-size-override":`calc(${b} - 8px)`,"--n-bezier":M,"--n-border-radius":z,"--n-border":ve,"--n-close-icon-size":Y,"--n-close-color-pressed":ye,"--n-close-color-hover":Re,"--n-close-border-radius":ae,"--n-close-icon-color":Se,"--n-close-icon-color-hover":be,"--n-close-icon-color-pressed":se,"--n-close-icon-color-disabled":Se,"--n-close-margin-top":we.top,"--n-close-margin-right":we.right,"--n-close-margin-bottom":we.bottom,"--n-close-margin-left":we.left,"--n-close-size":oe,"--n-color":k||(t.value?fe:P),"--n-color-checkable":I,"--n-color-checked":_,"--n-color-checked-hover":G,"--n-color-checked-pressed":q,"--n-color-hover-checkable":U,"--n-color-pressed-checkable":ue,"--n-font-size":pe,"--n-height":b,"--n-opacity-disabled":K,"--n-padding":m,"--n-text-color":x||J,"--n-text-color-checkable":j,"--n-text-color-checked":W,"--n-text-color-hover-checkable":T,"--n-text-color-pressed-checkable":F}}),v=r?We("tag",N(()=>{let g="";const{type:k,color:{color:x,textColor:S}={}}=e;return g+=k[0],g+=l.value[0],x&&(g+=`a${ot(x)}`),S&&(g+=`b${ot(S)}`),t.value&&(g+="c"),g}),O,e):void 0;return Object.assign(Object.assign({},c),{rtlEnabled:C,mergedClsPrefix:o,contentRef:n,mergedBordered:t,handleClick:f,handleCloseClick:d,cssVars:r?void 0:O,themeClass:v==null?void 0:v.themeClass,onRender:v==null?void 0:v.onRender})},render(){var e,n;const{mergedClsPrefix:t,rtlEnabled:o,closable:r,color:{borderColor:a}={},round:i,onRender:l,$slots:u}=this;l==null||l();const f=$e(u.avatar,c=>c&&h("div",{class:`${t}-tag__avatar`},c)),d=$e(u.icon,c=>c&&h("div",{class:`${t}-tag__icon`},c));return h("div",{class:[`${t}-tag`,this.themeClass,{[`${t}-tag--rtl`]:o,[`${t}-tag--strong`]:this.strong,[`${t}-tag--disabled`]:this.disabled,[`${t}-tag--checkable`]:this.checkable,[`${t}-tag--checked`]:this.checkable&&this.checked,[`${t}-tag--round`]:i,[`${t}-tag--avatar`]:f,[`${t}-tag--icon`]:d,[`${t}-tag--closable`]:r}],style:this.cssVars,onClick:this.handleClick,onMouseenter:this.onMouseenter,onMouseleave:this.onMouseleave},d||f,h("span",{class:`${t}-tag__content`,ref:"contentRef"},(n=(e=this.$slots).default)===null||n===void 0?void 0:n.call(e)),!this.checkable&&r?h(Yo,{clsPrefix:t,class:`${t}-tag__close`,disabled:this.disabled,onClick:this.handleCloseClick,focusable:this.internalCloseFocusable,round:i,isButtonTag:this.internalCloseIsButtonTag,absolute:!0}):null,!this.checkable&&this.mergedBordered?h("div",{class:`${t}-tag__border`,style:{borderColor:a}}):null)}}),xl=Q([A("base-selection",`
 --n-padding-single: var(--n-padding-single-top) var(--n-padding-single-right) var(--n-padding-single-bottom) var(--n-padding-single-left);
 --n-padding-multiple: var(--n-padding-multiple-top) var(--n-padding-multiple-right) var(--n-padding-multiple-bottom) var(--n-padding-multiple-left);
 position: relative;
 z-index: auto;
 box-shadow: none;
 width: 100%;
 max-width: 100%;
 display: inline-block;
 vertical-align: bottom;
 border-radius: var(--n-border-radius);
 min-height: var(--n-height);
 line-height: 1.5;
 font-size: var(--n-font-size);
 `,[A("base-loading",`
 color: var(--n-loading-color);
 `),A("base-selection-tags","min-height: var(--n-height);"),D("border, state-border",`
 position: absolute;
 left: 0;
 right: 0;
 top: 0;
 bottom: 0;
 pointer-events: none;
 border: var(--n-border);
 border-radius: inherit;
 transition:
 box-shadow .3s var(--n-bezier),
 border-color .3s var(--n-bezier);
 `),D("state-border",`
 z-index: 1;
 border-color: #0000;
 `),A("base-suffix",`
 cursor: pointer;
 position: absolute;
 top: 50%;
 transform: translateY(-50%);
 right: 10px;
 `,[D("arrow",`
 font-size: var(--n-arrow-size);
 color: var(--n-arrow-color);
 transition: color .3s var(--n-bezier);
 `)]),A("base-selection-overlay",`
 display: flex;
 align-items: center;
 white-space: nowrap;
 pointer-events: none;
 position: absolute;
 top: 0;
 right: 0;
 bottom: 0;
 left: 0;
 padding: var(--n-padding-single);
 transition: color .3s var(--n-bezier);
 `,[D("wrapper",`
 flex-basis: 0;
 flex-grow: 1;
 overflow: hidden;
 text-overflow: ellipsis;
 `)]),A("base-selection-placeholder",`
 color: var(--n-placeholder-color);
 `,[D("inner",`
 max-width: 100%;
 overflow: hidden;
 `)]),A("base-selection-tags",`
 cursor: pointer;
 outline: none;
 box-sizing: border-box;
 position: relative;
 z-index: auto;
 display: flex;
 padding: var(--n-padding-multiple);
 flex-wrap: wrap;
 align-items: center;
 width: 100%;
 vertical-align: bottom;
 background-color: var(--n-color);
 border-radius: inherit;
 transition:
 color .3s var(--n-bezier),
 box-shadow .3s var(--n-bezier),
 background-color .3s var(--n-bezier);
 `),A("base-selection-label",`
 height: var(--n-height);
 display: inline-flex;
 width: 100%;
 vertical-align: bottom;
 cursor: pointer;
 outline: none;
 z-index: auto;
 box-sizing: border-box;
 position: relative;
 transition:
 color .3s var(--n-bezier),
 box-shadow .3s var(--n-bezier),
 background-color .3s var(--n-bezier);
 border-radius: inherit;
 background-color: var(--n-color);
 align-items: center;
 `,[A("base-selection-input",`
 font-size: inherit;
 line-height: inherit;
 outline: none;
 cursor: pointer;
 box-sizing: border-box;
 border:none;
 width: 100%;
 padding: var(--n-padding-single);
 background-color: #0000;
 color: var(--n-text-color);
 transition: color .3s var(--n-bezier);
 caret-color: var(--n-caret-color);
 `,[D("content",`
 text-overflow: ellipsis;
 overflow: hidden;
 white-space: nowrap; 
 `)]),D("render-label",`
 color: var(--n-text-color);
 `)]),xe("disabled",[Q("&:hover",[D("state-border",`
 box-shadow: var(--n-box-shadow-hover);
 border: var(--n-border-hover);
 `)]),X("focus",[D("state-border",`
 box-shadow: var(--n-box-shadow-focus);
 border: var(--n-border-focus);
 `)]),X("active",[D("state-border",`
 box-shadow: var(--n-box-shadow-active);
 border: var(--n-border-active);
 `),A("base-selection-label","background-color: var(--n-color-active);"),A("base-selection-tags","background-color: var(--n-color-active);")])]),X("disabled","cursor: not-allowed;",[D("arrow",`
 color: var(--n-arrow-color-disabled);
 `),A("base-selection-label",`
 cursor: not-allowed;
 background-color: var(--n-color-disabled);
 `,[A("base-selection-input",`
 cursor: not-allowed;
 color: var(--n-text-color-disabled);
 `),D("render-label",`
 color: var(--n-text-color-disabled);
 `)]),A("base-selection-tags",`
 cursor: not-allowed;
 background-color: var(--n-color-disabled);
 `),A("base-selection-placeholder",`
 cursor: not-allowed;
 color: var(--n-placeholder-color-disabled);
 `)]),A("base-selection-input-tag",`
 height: calc(var(--n-height) - 6px);
 line-height: calc(var(--n-height) - 6px);
 outline: none;
 display: none;
 position: relative;
 margin-bottom: 3px;
 max-width: 100%;
 vertical-align: bottom;
 `,[D("input",`
 font-size: inherit;
 font-family: inherit;
 min-width: 1px;
 padding: 0;
 background-color: #0000;
 outline: none;
 border: none;
 max-width: 100%;
 overflow: hidden;
 width: 1em;
 line-height: inherit;
 cursor: pointer;
 color: var(--n-text-color);
 caret-color: var(--n-caret-color);
 `),D("mirror",`
 position: absolute;
 left: 0;
 top: 0;
 white-space: pre;
 visibility: hidden;
 user-select: none;
 -webkit-user-select: none;
 opacity: 0;
 `)]),["warning","error"].map(e=>X(`${e}-status`,[D("state-border",`border: var(--n-border-${e});`),xe("disabled",[Q("&:hover",[D("state-border",`
 box-shadow: var(--n-box-shadow-hover-${e});
 border: var(--n-border-hover-${e});
 `)]),X("active",[D("state-border",`
 box-shadow: var(--n-box-shadow-active-${e});
 border: var(--n-border-active-${e});
 `),A("base-selection-label",`background-color: var(--n-color-active-${e});`),A("base-selection-tags",`background-color: var(--n-color-active-${e});`)]),X("focus",[D("state-border",`
 box-shadow: var(--n-box-shadow-focus-${e});
 border: var(--n-border-focus-${e});
 `)])])]))]),A("base-selection-popover",`
 margin-bottom: -3px;
 display: flex;
 flex-wrap: wrap;
 margin-right: -8px;
 `),A("base-selection-tag-wrapper",`
 max-width: 100%;
 display: inline-flex;
 padding: 0 7px 3px 0;
 `,[Q("&:last-child","padding-right: 0;"),A("tag",`
 font-size: 14px;
 max-width: 100%;
 `,[D("content",`
 line-height: 1.25;
 text-overflow: ellipsis;
 overflow: hidden;
 `)])])]),Sl=he({name:"InternalSelection",props:Object.assign(Object.assign({},ce.props),{clsPrefix:{type:String,required:!0},bordered:{type:Boolean,default:void 0},active:Boolean,pattern:{type:String,default:""},placeholder:String,selectedOption:{type:Object,default:null},selectedOptions:{type:Array,default:null},labelField:{type:String,default:"label"},valueField:{type:String,default:"value"},multiple:Boolean,filterable:Boolean,clearable:Boolean,disabled:Boolean,size:{type:String,default:"medium"},loading:Boolean,autofocus:Boolean,showArrow:{type:Boolean,default:!0},inputProps:Object,focused:Boolean,renderTag:Function,onKeydown:Function,onClick:Function,onBlur:Function,onFocus:Function,onDeleteOption:Function,maxTagCount:[String,Number],ellipsisTagPopoverProps:Object,onClear:Function,onPatternInput:Function,onPatternFocus:Function,onPatternBlur:Function,renderLabel:Function,status:String,inlineThemeDisabled:Boolean,ignoreComposition:{type:Boolean,default:!0},onResize:Function}),setup(e){const{mergedClsPrefixRef:n,mergedRtlRef:t}=Ae(e),o=hn("InternalSelection",t,n),r=E(null),a=E(null),i=E(null),l=E(null),u=E(null),f=E(null),d=E(null),c=E(null),C=E(null),O=E(null),v=E(!1),g=E(!1),k=E(!1),x=ce("InternalSelection","-internal-selection",xl,er,e,te(e,"clsPrefix")),S=N(()=>e.clearable&&!e.disabled&&(k.value||e.active)),M=N(()=>e.selectedOption?e.renderTag?e.renderTag({option:e.selectedOption,handleClose:()=>{}}):e.renderLabel?e.renderLabel(e.selectedOption,!0):Le(e.selectedOption[e.labelField],e.selectedOption,!0):e.placeholder),m=N(()=>{const p=e.selectedOption;if(p)return p[e.labelField]}),y=N(()=>e.multiple?!!(Array.isArray(e.selectedOptions)&&e.selectedOptions.length):e.selectedOption!==null);function z(){var p;const{value:R}=r;if(R){const{value:ne}=a;ne&&(ne.style.width=`${R.offsetWidth}px`,e.maxTagCount!=="responsive"&&((p=C.value)===null||p===void 0||p.sync({showAllItemsBeforeCalculate:!1})))}}function K(){const{value:p}=O;p&&(p.style.display="none")}function j(){const{value:p}=O;p&&(p.style.display="inline-block")}_e(te(e,"active"),p=>{p||K()}),_e(te(e,"pattern"),()=>{e.multiple&&Dn(z)});function T(p){const{onFocus:R}=e;R&&R(p)}function F(p){const{onBlur:R}=e;R&&R(p)}function W(p){const{onDeleteOption:R}=e;R&&R(p)}function I(p){const{onClear:R}=e;R&&R(p)}function U(p){const{onPatternInput:R}=e;R&&R(p)}function ue(p){var R;(!p.relatedTarget||!(!((R=i.value)===null||R===void 0)&&R.contains(p.relatedTarget)))&&T(p)}function _(p){var R;!((R=i.value)===null||R===void 0)&&R.contains(p.relatedTarget)||F(p)}function G(p){I(p)}function q(){k.value=!0}function ae(){k.value=!1}function le(p){!e.active||!e.filterable||p.target!==a.value&&p.preventDefault()}function fe(p){W(p)}const oe=E(!1);function Y(p){if(p.key==="Backspace"&&!oe.value&&!e.pattern.length){const{selectedOptions:R}=e;R!=null&&R.length&&fe(R[R.length-1])}}let pe=null;function b(p){const{value:R}=r;if(R){const ne=p.target.value;R.textContent=ne,z()}e.ignoreComposition&&oe.value?pe=p:U(p)}function P(){oe.value=!0}function J(){oe.value=!1,e.ignoreComposition&&U(pe),pe=null}function ve(p){var R;g.value=!0,(R=e.onPatternFocus)===null||R===void 0||R.call(e,p)}function Se(p){var R;g.value=!1,(R=e.onPatternBlur)===null||R===void 0||R.call(e,p)}function be(){var p,R;if(e.filterable)g.value=!1,(p=f.value)===null||p===void 0||p.blur(),(R=a.value)===null||R===void 0||R.blur();else if(e.multiple){const{value:ne}=l;ne==null||ne.blur()}else{const{value:ne}=u;ne==null||ne.blur()}}function se(){var p,R,ne;e.filterable?(g.value=!1,(p=f.value)===null||p===void 0||p.focus()):e.multiple?(R=l.value)===null||R===void 0||R.focus():(ne=u.value)===null||ne===void 0||ne.focus()}function Re(){const{value:p}=a;p&&(j(),p.focus())}function ye(){const{value:p}=a;p&&p.blur()}function we(p){const{value:R}=d;R&&R.setTextContent(`+${p}`)}function je(){const{value:p}=c;return p}function Ve(){return a.value}let Me=null;function ze(){Me!==null&&window.clearTimeout(Me)}function Ue(){e.active||(ze(),Me=window.setTimeout(()=>{y.value&&(v.value=!0)},100))}function Ge(){ze()}function qe(p){p||(ze(),v.value=!1)}_e(y,p=>{p||(v.value=!1)}),cn(()=>{Vn(()=>{const p=f.value;p&&(e.disabled?p.removeAttribute("tabindex"):p.tabIndex=g.value?-1:0)})}),Ft(i,e.onResize);const{inlineThemeDisabled:Be}=e,Ee=N(()=>{const{size:p}=e,{common:{cubicBezierEaseInOut:R},self:{fontWeight:ne,borderRadius:vn,color:gn,placeholderColor:pn,textColor:Ye,paddingSingle:Je,paddingMultiple:Qe,caretColor:bn,colorDisabled:mn,textColorDisabled:en,placeholderColorDisabled:ke,colorActive:s,boxShadowFocus:w,boxShadowActive:$,boxShadowHover:H,border:B,borderFocus:L,borderHover:V,borderActive:re,arrowColor:Ce,arrowColorDisabled:Wt,loadingColor:jt,colorActiveWarning:Vt,boxShadowFocusWarning:Ut,boxShadowActiveWarning:Gt,boxShadowHoverWarning:qt,borderWarning:Zt,borderFocusWarning:Xt,borderHoverWarning:Yt,borderActiveWarning:Jt,colorActiveError:Qt,boxShadowFocusError:eo,boxShadowActiveError:no,boxShadowHoverError:to,borderError:oo,borderFocusError:ro,borderHoverError:io,borderActiveError:lo,clearColor:ao,clearColorHover:so,clearColorPressed:co,clearSize:uo,arrowSize:fo,[ee("height",p)]:ho,[ee("fontSize",p)]:vo}}=x.value,nn=De(Je),tn=De(Qe);return{"--n-bezier":R,"--n-border":B,"--n-border-active":re,"--n-border-focus":L,"--n-border-hover":V,"--n-border-radius":vn,"--n-box-shadow-active":$,"--n-box-shadow-focus":w,"--n-box-shadow-hover":H,"--n-caret-color":bn,"--n-color":gn,"--n-color-active":s,"--n-color-disabled":mn,"--n-font-size":vo,"--n-height":ho,"--n-padding-single-top":nn.top,"--n-padding-multiple-top":tn.top,"--n-padding-single-right":nn.right,"--n-padding-multiple-right":tn.right,"--n-padding-single-left":nn.left,"--n-padding-multiple-left":tn.left,"--n-padding-single-bottom":nn.bottom,"--n-padding-multiple-bottom":tn.bottom,"--n-placeholder-color":pn,"--n-placeholder-color-disabled":ke,"--n-text-color":Ye,"--n-text-color-disabled":en,"--n-arrow-color":Ce,"--n-arrow-color-disabled":Wt,"--n-loading-color":jt,"--n-color-active-warning":Vt,"--n-box-shadow-focus-warning":Ut,"--n-box-shadow-active-warning":Gt,"--n-box-shadow-hover-warning":qt,"--n-border-warning":Zt,"--n-border-focus-warning":Xt,"--n-border-hover-warning":Yt,"--n-border-active-warning":Jt,"--n-color-active-error":Qt,"--n-box-shadow-focus-error":eo,"--n-box-shadow-active-error":no,"--n-box-shadow-hover-error":to,"--n-border-error":oo,"--n-border-focus-error":ro,"--n-border-hover-error":io,"--n-border-active-error":lo,"--n-clear-size":uo,"--n-clear-color":ao,"--n-clear-color-hover":so,"--n-clear-color-pressed":co,"--n-arrow-size":fo,"--n-font-weight":ne}}),me=Be?We("internal-selection",N(()=>e.size[0]),Ee,e):void 0;return{mergedTheme:x,mergedClearable:S,mergedClsPrefix:n,rtlEnabled:o,patternInputFocused:g,filterablePlaceholder:M,label:m,selected:y,showTagsPanel:v,isComposing:oe,counterRef:d,counterWrapperRef:c,patternInputMirrorRef:r,patternInputRef:a,selfRef:i,multipleElRef:l,singleElRef:u,patternInputWrapperRef:f,overflowRef:C,inputTagElRef:O,handleMouseDown:le,handleFocusin:ue,handleClear:G,handleMouseEnter:q,handleMouseLeave:ae,handleDeleteOption:fe,handlePatternKeyDown:Y,handlePatternInputInput:b,handlePatternInputBlur:Se,handlePatternInputFocus:ve,handleMouseEnterCounter:Ue,handleMouseLeaveCounter:Ge,handleFocusout:_,handleCompositionEnd:J,handleCompositionStart:P,onPopoverUpdateShow:qe,focus:se,focusInput:Re,blur:be,blurInput:ye,updateCounter:we,getCounter:je,getTail:Ve,renderLabel:e.renderLabel,cssVars:Be?void 0:Ee,themeClass:me==null?void 0:me.themeClass,onRender:me==null?void 0:me.onRender}},render(){const{status:e,multiple:n,size:t,disabled:o,filterable:r,maxTagCount:a,bordered:i,clsPrefix:l,ellipsisTagPopoverProps:u,onRender:f,renderTag:d,renderLabel:c}=this;f==null||f();const C=a==="responsive",O=typeof a=="number",v=C||O,g=h(Qo,null,{default:()=>h(cr,{clsPrefix:l,loading:this.loading,showArrow:this.showArrow,showClear:this.mergedClearable&&this.selected,onClear:this.handleClear},{default:()=>{var x,S;return(S=(x=this.$slots).arrow)===null||S===void 0?void 0:S.call(x)}})});let k;if(n){const{labelField:x}=this,S=U=>h("div",{class:`${l}-base-selection-tag-wrapper`,key:U.value},d?d({option:U,handleClose:()=>{this.handleDeleteOption(U)}}):h(Mn,{size:t,closable:!U.disabled,disabled:o,onClose:()=>{this.handleDeleteOption(U)},internalCloseIsButtonTag:!1,internalCloseFocusable:!1},{default:()=>c?c(U,!0):Le(U[x],U,!0)})),M=()=>(O?this.selectedOptions.slice(0,a):this.selectedOptions).map(S),m=r?h("div",{class:`${l}-base-selection-input-tag`,ref:"inputTagElRef",key:"__input-tag__"},h("input",Object.assign({},this.inputProps,{ref:"patternInputRef",tabindex:-1,disabled:o,value:this.pattern,autofocus:this.autofocus,class:`${l}-base-selection-input-tag__input`,onBlur:this.handlePatternInputBlur,onFocus:this.handlePatternInputFocus,onKeydown:this.handlePatternKeyDown,onInput:this.handlePatternInputInput,onCompositionstart:this.handleCompositionStart,onCompositionend:this.handleCompositionEnd})),h("span",{ref:"patternInputMirrorRef",class:`${l}-base-selection-input-tag__mirror`},this.pattern)):null,y=C?()=>h("div",{class:`${l}-base-selection-tag-wrapper`,ref:"counterWrapperRef"},h(Mn,{size:t,ref:"counterRef",onMouseenter:this.handleMouseEnterCounter,onMouseleave:this.handleMouseLeaveCounter,disabled:o})):void 0;let z;if(O){const U=this.selectedOptions.length-a;U>0&&(z=h("div",{class:`${l}-base-selection-tag-wrapper`,key:"__counter__"},h(Mn,{size:t,ref:"counterRef",onMouseenter:this.handleMouseEnterCounter,disabled:o},{default:()=>`+${U}`})))}const K=C?r?h(rt,{ref:"overflowRef",updateCounter:this.updateCounter,getCounter:this.getCounter,getTail:this.getTail,style:{width:"100%",display:"flex",overflow:"hidden"}},{default:M,counter:y,tail:()=>m}):h(rt,{ref:"overflowRef",updateCounter:this.updateCounter,getCounter:this.getCounter,style:{width:"100%",display:"flex",overflow:"hidden"}},{default:M,counter:y}):O&&z?M().concat(z):M(),j=v?()=>h("div",{class:`${l}-base-selection-popover`},C?M():this.selectedOptions.map(S)):void 0,T=v?Object.assign({show:this.showTagsPanel,trigger:"hover",overlap:!0,placement:"top",width:"trigger",onUpdateShow:this.onPopoverUpdateShow,theme:this.mergedTheme.peers.Popover,themeOverrides:this.mergedTheme.peerOverrides.Popover},u):null,W=(this.selected?!1:this.active?!this.pattern&&!this.isComposing:!0)?h("div",{class:`${l}-base-selection-placeholder ${l}-base-selection-overlay`},h("div",{class:`${l}-base-selection-placeholder__inner`},this.placeholder)):null,I=r?h("div",{ref:"patternInputWrapperRef",class:`${l}-base-selection-tags`},K,C?null:m,g):h("div",{ref:"multipleElRef",class:`${l}-base-selection-tags`,tabindex:o?void 0:0},K,g);k=h(Ot,null,v?h(Ht,Object.assign({},T,{scrollable:!0,style:"max-height: calc(var(--v-target-height) * 6.6);"}),{trigger:()=>I,default:j}):I,W)}else if(r){const x=this.pattern||this.isComposing,S=this.active?!x:!this.selected,M=this.active?!1:this.selected;k=h("div",{ref:"patternInputWrapperRef",class:`${l}-base-selection-label`,title:this.patternInputFocused?void 0:it(this.label)},h("input",Object.assign({},this.inputProps,{ref:"patternInputRef",class:`${l}-base-selection-input`,value:this.active?this.pattern:"",placeholder:"",readonly:o,disabled:o,tabindex:-1,autofocus:this.autofocus,onFocus:this.handlePatternInputFocus,onBlur:this.handlePatternInputBlur,onInput:this.handlePatternInputInput,onCompositionstart:this.handleCompositionStart,onCompositionend:this.handleCompositionEnd})),M?h("div",{class:`${l}-base-selection-label__render-label ${l}-base-selection-overlay`,key:"input"},h("div",{class:`${l}-base-selection-overlay__wrapper`},d?d({option:this.selectedOption,handleClose:()=>{}}):c?c(this.selectedOption,!0):Le(this.label,this.selectedOption,!0))):null,S?h("div",{class:`${l}-base-selection-placeholder ${l}-base-selection-overlay`,key:"placeholder"},h("div",{class:`${l}-base-selection-overlay__wrapper`},this.filterablePlaceholder)):null,g)}else k=h("div",{ref:"singleElRef",class:`${l}-base-selection-label`,tabindex:this.disabled?void 0:0},this.label!==void 0?h("div",{class:`${l}-base-selection-input`,title:it(this.label),key:"input"},h("div",{class:`${l}-base-selection-input__content`},d?d({option:this.selectedOption,handleClose:()=>{}}):c?c(this.selectedOption,!0):Le(this.label,this.selectedOption,!0))):h("div",{class:`${l}-base-selection-placeholder ${l}-base-selection-overlay`,key:"placeholder"},h("div",{class:`${l}-base-selection-placeholder__inner`},this.placeholder)),g);return h("div",{ref:"selfRef",class:[`${l}-base-selection`,this.rtlEnabled&&`${l}-base-selection--rtl`,this.themeClass,e&&`${l}-base-selection--${e}-status`,{[`${l}-base-selection--active`]:this.active,[`${l}-base-selection--selected`]:this.selected||this.active&&this.pattern,[`${l}-base-selection--disabled`]:this.disabled,[`${l}-base-selection--multiple`]:this.multiple,[`${l}-base-selection--focus`]:this.focused}],style:this.cssVars,onClick:this.onClick,onMouseenter:this.handleMouseEnter,onMouseleave:this.handleMouseLeave,onKeydown:this.onKeydown,onFocusin:this.handleFocusin,onFocusout:this.handleFocusout,onMousedown:this.handleMouseDown},k,i?h("div",{class:`${l}-base-selection__border`}):null,i?h("div",{class:`${l}-base-selection__state-border`}):null)}});function dn(e){return e.type==="group"}function Kt(e){return e.type==="ignored"}function zn(e,n){try{return!!(1+n.toString().toLowerCase().indexOf(e.trim().toLowerCase()))}catch{return!1}}function Ol(e,n){return{getIsGroup:dn,getIgnored:Kt,getKey(o){return dn(o)?o.name||o.key||"key-required":o[e]},getChildren(o){return o[n]}}}function Pl(e,n,t,o){if(!n)return e;function r(a){if(!Array.isArray(a))return[];const i=[];for(const l of a)if(dn(l)){const u=r(l[o]);u.length&&i.push(Object.assign({},l,{[o]:u}))}else{if(Kt(l))continue;n(t,l)&&i.push(l)}return i}return r(e)}function Rl(e,n,t){const o=new Map;return e.forEach(r=>{dn(r)?r[t].forEach(a=>{o.set(a[n],a)}):o.set(r[n],r)}),o}const kl=Q([A("select",`
 z-index: auto;
 outline: none;
 width: 100%;
 position: relative;
 font-weight: var(--n-font-weight);
 `),A("select-menu",`
 margin: 4px 0;
 box-shadow: var(--n-menu-box-shadow);
 `,[St({originalTransition:"background-color .3s var(--n-bezier), box-shadow .3s var(--n-bezier)"})])]),Tl=Object.assign(Object.assign({},ce.props),{to:Ie.propTo,bordered:{type:Boolean,default:void 0},clearable:Boolean,clearCreatedOptionsOnClear:{type:Boolean,default:!0},clearFilterAfterSelect:{type:Boolean,default:!0},options:{type:Array,default:()=>[]},defaultValue:{type:[String,Number,Array],default:null},keyboard:{type:Boolean,default:!0},value:[String,Number,Array],placeholder:String,menuProps:Object,multiple:Boolean,size:String,menuSize:{type:String},filterable:Boolean,disabled:{type:Boolean,default:void 0},remote:Boolean,loading:Boolean,filter:Function,placement:{type:String,default:"bottom-start"},widthMode:{type:String,default:"trigger"},tag:Boolean,onCreate:Function,fallbackOption:{type:[Function,Boolean],default:void 0},show:{type:Boolean,default:void 0},showArrow:{type:Boolean,default:!0},maxTagCount:[Number,String],ellipsisTagPopoverProps:Object,consistentMenuWidth:{type:Boolean,default:!0},virtualScroll:{type:Boolean,default:!0},labelField:{type:String,default:"label"},valueField:{type:String,default:"value"},childrenField:{type:String,default:"children"},renderLabel:Function,renderOption:Function,renderTag:Function,"onUpdate:value":[Function,Array],inputProps:Object,nodeProps:Function,ignoreComposition:{type:Boolean,default:!0},showOnFocus:Boolean,onUpdateValue:[Function,Array],onBlur:[Function,Array],onClear:[Function,Array],onFocus:[Function,Array],onScroll:[Function,Array],onSearch:[Function,Array],onUpdateShow:[Function,Array],"onUpdate:show":[Function,Array],displayDirective:{type:String,default:"show"},resetMenuOnOptionsChange:{type:Boolean,default:!0},status:String,showCheckmark:{type:Boolean,default:!0},scrollbarProps:Object,onChange:[Function,Array],items:Array}),Bl=he({name:"Select",props:Tl,slots:Object,setup(e){const{mergedClsPrefixRef:n,mergedBorderedRef:t,namespaceRef:o,inlineThemeDisabled:r,mergedComponentPropsRef:a}=Ae(e),i=ce("Select","-select",kl,or,e,n),l=E(e.defaultValue),u=te(e,"value"),f=In(u,l),d=E(!1),c=E(""),C=$t(e,["items","options"]),O=E([]),v=E([]),g=N(()=>v.value.concat(O.value).concat(C.value)),k=N(()=>{const{filter:s}=e;if(s)return s;const{labelField:w,valueField:$}=e;return(H,B)=>{if(!B)return!1;const L=B[w];if(typeof L=="string")return zn(H,L);const V=B[$];return typeof V=="string"?zn(H,V):typeof V=="number"?zn(H,String(V)):!1}}),x=N(()=>{if(e.remote)return C.value;{const{value:s}=g,{value:w}=c;return!w.length||!e.filterable?s:Pl(s,k.value,w,e.childrenField)}}),S=N(()=>{const{valueField:s,childrenField:w}=e,$=Ol(s,w);return tl(x.value,$)}),M=N(()=>Rl(g.value,e.valueField,e.childrenField)),m=E(!1),y=In(te(e,"show"),m),z=E(null),K=E(null),j=E(null),{localeRef:T}=Mt("Select"),F=N(()=>{var s;return(s=e.placeholder)!==null&&s!==void 0?s:T.value.placeholder}),W=[],I=E(new Map),U=N(()=>{const{fallbackOption:s}=e;if(s===void 0){const{labelField:w,valueField:$}=e;return H=>({[w]:String(H),[$]:H})}return s===!1?!1:w=>Object.assign(s(w),{value:w})});function ue(s){const w=e.remote,{value:$}=I,{value:H}=M,{value:B}=U,L=[];return s.forEach(V=>{if(H.has(V))L.push(H.get(V));else if(w&&$.has(V))L.push($.get(V));else if(B){const re=B(V);re&&L.push(re)}}),L}const _=N(()=>{if(e.multiple){const{value:s}=f;return Array.isArray(s)?ue(s):[]}return null}),G=N(()=>{const{value:s}=f;return!e.multiple&&!Array.isArray(s)?s===null?null:ue([s])[0]||null:null}),q=nr(e,{mergedSize:s=>{var w,$;const{size:H}=e;if(H)return H;const{mergedSize:B}=s||{};if(B!=null&&B.value)return B.value;const L=($=(w=a==null?void 0:a.value)===null||w===void 0?void 0:w.Select)===null||$===void 0?void 0:$.size;return L||"medium"}}),{mergedSizeRef:ae,mergedDisabledRef:le,mergedStatusRef:fe}=q;function oe(s,w){const{onChange:$,"onUpdate:value":H,onUpdateValue:B}=e,{nTriggerFormChange:L,nTriggerFormInput:V}=q;$&&de($,s,w),B&&de(B,s,w),H&&de(H,s,w),l.value=s,L(),V()}function Y(s){const{onBlur:w}=e,{nTriggerFormBlur:$}=q;w&&de(w,s),$()}function pe(){const{onClear:s}=e;s&&de(s)}function b(s){const{onFocus:w,showOnFocus:$}=e,{nTriggerFormFocus:H}=q;w&&de(w,s),H(),$&&be()}function P(s){const{onSearch:w}=e;w&&de(w,s)}function J(s){const{onScroll:w}=e;w&&de(w,s)}function ve(){var s;const{remote:w,multiple:$}=e;if(w){const{value:H}=I;if($){const{valueField:B}=e;(s=_.value)===null||s===void 0||s.forEach(L=>{H.set(L[B],L)})}else{const B=G.value;B&&H.set(B[e.valueField],B)}}}function Se(s){const{onUpdateShow:w,"onUpdate:show":$}=e;w&&de(w,s),$&&de($,s),m.value=s}function be(){le.value||(Se(!0),m.value=!0,e.filterable&&Qe())}function se(){Se(!1)}function Re(){c.value="",v.value=W}const ye=E(!1);function we(){e.filterable&&(ye.value=!0)}function je(){e.filterable&&(ye.value=!1,y.value||Re())}function Ve(){le.value||(y.value?e.filterable?Qe():se():be())}function Me(s){var w,$;!(($=(w=j.value)===null||w===void 0?void 0:w.selfRef)===null||$===void 0)&&$.contains(s.relatedTarget)||(d.value=!1,Y(s),se())}function ze(s){b(s),d.value=!0}function Ue(){d.value=!0}function Ge(s){var w;!((w=z.value)===null||w===void 0)&&w.$el.contains(s.relatedTarget)||(d.value=!1,Y(s),se())}function qe(){var s;(s=z.value)===null||s===void 0||s.focus(),se()}function Be(s){var w;y.value&&(!((w=z.value)===null||w===void 0)&&w.$el.contains(Fn(s))||se())}function Ee(s){if(!Array.isArray(s))return[];if(U.value)return Array.from(s);{const{remote:w}=e,{value:$}=M;if(w){const{value:H}=I;return s.filter(B=>$.has(B)||H.has(B))}else return s.filter(H=>$.has(H))}}function me(s){p(s.rawNode)}function p(s){if(le.value)return;const{tag:w,remote:$,clearFilterAfterSelect:H,valueField:B}=e;if(w&&!$){const{value:L}=v,V=L[0]||null;if(V){const re=O.value;re.length?re.push(V):O.value=[V],v.value=W}}if($&&I.value.set(s[B],s),e.multiple){const L=Ee(f.value),V=L.findIndex(re=>re===s[B]);if(~V){if(L.splice(V,1),w&&!$){const re=R(s[B]);~re&&(O.value.splice(re,1),H&&(c.value=""))}}else L.push(s[B]),H&&(c.value="");oe(L,ue(L))}else{if(w&&!$){const L=R(s[B]);~L?O.value=[O.value[L]]:O.value=W}Je(),se(),oe(s[B],s)}}function R(s){return O.value.findIndex($=>$[e.valueField]===s)}function ne(s){y.value||be();const{value:w}=s.target;c.value=w;const{tag:$,remote:H}=e;if(P(w),$&&!H){if(!w){v.value=W;return}const{onCreate:B}=e,L=B?B(w):{[e.labelField]:w,[e.valueField]:w},{valueField:V,labelField:re}=e;C.value.some(Ce=>Ce[V]===L[V]||Ce[re]===L[re])||O.value.some(Ce=>Ce[V]===L[V]||Ce[re]===L[re])?v.value=W:v.value=[L]}}function vn(s){s.stopPropagation();const{multiple:w,tag:$,remote:H,clearCreatedOptionsOnClear:B}=e;!w&&e.filterable&&se(),$&&!H&&B&&(O.value=W),pe(),w?oe([],[]):oe(null,null)}function gn(s){!Xe(s,"action")&&!Xe(s,"empty")&&!Xe(s,"header")&&s.preventDefault()}function pn(s){J(s)}function Ye(s){var w,$,H,B,L;if(!e.keyboard){s.preventDefault();return}switch(s.key){case" ":if(e.filterable)break;s.preventDefault();case"Enter":if(!(!((w=z.value)===null||w===void 0)&&w.isComposing)){if(y.value){const V=($=j.value)===null||$===void 0?void 0:$.getPendingTmNode();V?me(V):e.filterable||(se(),Je())}else if(be(),e.tag&&ye.value){const V=v.value[0];if(V){const re=V[e.valueField],{value:Ce}=f;e.multiple&&Array.isArray(Ce)&&Ce.includes(re)||p(V)}}}s.preventDefault();break;case"ArrowUp":if(s.preventDefault(),e.loading)return;y.value&&((H=j.value)===null||H===void 0||H.prev());break;case"ArrowDown":if(s.preventDefault(),e.loading)return;y.value?(B=j.value)===null||B===void 0||B.next():be();break;case"Escape":y.value&&(tr(s),se()),(L=z.value)===null||L===void 0||L.focus();break}}function Je(){var s;(s=z.value)===null||s===void 0||s.focus()}function Qe(){var s;(s=z.value)===null||s===void 0||s.focusInput()}function bn(){var s;y.value&&((s=K.value)===null||s===void 0||s.syncPosition())}ve(),_e(te(e,"options"),ve);const mn={focus:()=>{var s;(s=z.value)===null||s===void 0||s.focus()},focusInput:()=>{var s;(s=z.value)===null||s===void 0||s.focusInput()},blur:()=>{var s;(s=z.value)===null||s===void 0||s.blur()},blurInput:()=>{var s;(s=z.value)===null||s===void 0||s.blurInput()}},en=N(()=>{const{self:{menuBoxShadow:s}}=i.value;return{"--n-menu-box-shadow":s}}),ke=r?We("select",void 0,en,e):void 0;return Object.assign(Object.assign({},mn),{mergedStatus:fe,mergedClsPrefix:n,mergedBordered:t,namespace:o,treeMate:S,isMounted:Rt(),triggerRef:z,menuRef:j,pattern:c,uncontrolledShow:m,mergedShow:y,adjustedTo:Ie(e),uncontrolledValue:l,mergedValue:f,followerRef:K,localizedPlaceholder:F,selectedOption:G,selectedOptions:_,mergedSize:ae,mergedDisabled:le,focused:d,activeWithoutMenuOpen:ye,inlineThemeDisabled:r,onTriggerInputFocus:we,onTriggerInputBlur:je,handleTriggerOrMenuResize:bn,handleMenuFocus:Ue,handleMenuBlur:Ge,handleMenuTabOut:qe,handleTriggerClick:Ve,handleToggle:me,handleDeleteOption:p,handlePatternInput:ne,handleClear:vn,handleTriggerBlur:Me,handleTriggerFocus:ze,handleKeydown:Ye,handleMenuAfterLeave:Re,handleMenuClickOutside:Be,handleMenuScroll:pn,handleMenuKeydown:Ye,handleMenuMousedown:gn,mergedTheme:i,cssVars:r?void 0:en,themeClass:ke==null?void 0:ke.themeClass,onRender:ke==null?void 0:ke.onRender})},render(){return h("div",{class:`${this.mergedClsPrefix}-select`},h(Tt,null,{default:()=>[h(_t,null,{default:()=>h(Sl,{ref:"triggerRef",inlineThemeDisabled:this.inlineThemeDisabled,status:this.mergedStatus,inputProps:this.inputProps,clsPrefix:this.mergedClsPrefix,showArrow:this.showArrow,maxTagCount:this.maxTagCount,ellipsisTagPopoverProps:this.ellipsisTagPopoverProps,bordered:this.mergedBordered,active:this.activeWithoutMenuOpen||this.mergedShow,pattern:this.pattern,placeholder:this.localizedPlaceholder,selectedOption:this.selectedOption,selectedOptions:this.selectedOptions,multiple:this.multiple,renderTag:this.renderTag,renderLabel:this.renderLabel,filterable:this.filterable,clearable:this.clearable,disabled:this.mergedDisabled,size:this.mergedSize,theme:this.mergedTheme.peers.InternalSelection,labelField:this.labelField,valueField:this.valueField,themeOverrides:this.mergedTheme.peerOverrides.InternalSelection,loading:this.loading,focused:this.focused,onClick:this.handleTriggerClick,onDeleteOption:this.handleDeleteOption,onPatternInput:this.handlePatternInput,onClear:this.handleClear,onBlur:this.handleTriggerBlur,onFocus:this.handleTriggerFocus,onKeydown:this.handleKeydown,onPatternBlur:this.onTriggerInputBlur,onPatternFocus:this.onTriggerInputFocus,onResize:this.handleTriggerOrMenuResize,ignoreComposition:this.ignoreComposition},{arrow:()=>{var e,n;return[(n=(e=this.$slots).arrow)===null||n===void 0?void 0:n.call(e)]}})}),h(kt,{ref:"followerRef",show:this.mergedShow,to:this.adjustedTo,teleportDisabled:this.adjustedTo===Ie.tdkey,containerClass:this.namespace,width:this.consistentMenuWidth?"target":void 0,minWidth:"target",placement:this.placement},{default:()=>h(jn,{name:"fade-in-scale-up-transition",appear:this.isMounted,onAfterLeave:this.handleMenuAfterLeave},{default:()=>{var e,n,t;return this.mergedShow||this.displayDirective==="show"?((e=this.onRender)===null||e===void 0||e.call(this),Un(h(sl,Object.assign({},this.menuProps,{ref:"menuRef",onResize:this.handleTriggerOrMenuResize,inlineThemeDisabled:this.inlineThemeDisabled,virtualScroll:this.consistentMenuWidth&&this.virtualScroll,class:[`${this.mergedClsPrefix}-select-menu`,this.themeClass,(n=this.menuProps)===null||n===void 0?void 0:n.class],clsPrefix:this.mergedClsPrefix,focusable:!0,labelField:this.labelField,valueField:this.valueField,autoPending:!0,nodeProps:this.nodeProps,theme:this.mergedTheme.peers.InternalSelectMenu,themeOverrides:this.mergedTheme.peerOverrides.InternalSelectMenu,treeMate:this.treeMate,multiple:this.multiple,size:this.menuSize,renderOption:this.renderOption,renderLabel:this.renderLabel,value:this.mergedValue,style:[(t=this.menuProps)===null||t===void 0?void 0:t.style,this.cssVars],onToggle:this.handleToggle,onScroll:this.handleMenuScroll,onFocus:this.handleMenuFocus,onBlur:this.handleMenuBlur,onKeydown:this.handleMenuKeydown,onTabOut:this.handleMenuTabOut,onMousedown:this.handleMenuMousedown,show:this.mergedShow,showCheckmark:this.showCheckmark,resetMenuOnOptionsChange:this.resetMenuOnOptionsChange,scrollbarProps:this.scrollbarProps}),{empty:()=>{var o,r;return[(r=(o=this.$slots).empty)===null||r===void 0?void 0:r.call(o)]},header:()=>{var o,r;return[(r=(o=this.$slots).header)===null||r===void 0?void 0:r.call(o)]},action:()=>{var o,r;return[(r=(o=this.$slots).action)===null||r===void 0?void 0:r.call(o)]}}),this.displayDirective==="show"?[[Pt,this.mergedShow],[ln,this.handleMenuClickOutside,void 0,{capture:!0}]]:[[ln,this.handleMenuClickOutside,void 0,{capture:!0}]])):null}})})]}))}}),_l=Object.assign(Object.assign({},Dt),ce.props),El=he({name:"Tooltip",props:_l,slots:Object,__popover__:!0,setup(e){const{mergedClsPrefixRef:n}=Ae(e),t=ce("Tooltip","-tooltip",void 0,rr,e,n),o=E(null);return Object.assign(Object.assign({},{syncPosition(){o.value.syncPosition()},setShow(a){o.value.setShow(a)}}),{popoverRef:o,mergedTheme:t,popoverThemeOverrides:N(()=>t.value.self)})},render(){const{mergedTheme:e,internalExtraClass:n}=this;return h(Ht,Object.assign(Object.assign({},this.$props),{theme:e.peers.Popover,themeOverrides:e.peerOverrides.Popover,builtinThemeOverrides:this.popoverThemeOverrides,internalExtraClass:n.concat("tooltip"),ref:"popoverRef"}),this.$slots)}});export{il as N,Mn as _,sl as a,Ht as b,Bl as c,El as d,ml as e,Ol as f,tl as g,Il as h,Al as i,On as m,Dt as p,cl as r,bl as t,$t as u};

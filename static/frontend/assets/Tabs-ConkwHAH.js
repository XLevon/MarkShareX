import{aA as G,by as l,cK as L,dV as xt,c2 as yt,bY as ee,d4 as St,G as y,H as n,aY as wt,P as s,e as Ct,l as Rt,dK as _e,dX as te,d8 as zt,dY as We,a9 as F,e5 as Ee,aV as Tt,cF as $t,al as A,ak as Pt,bJ as Le,dd as _t,c9 as Wt,b as Et,d as Lt,cO as kt,N as Bt,cn as At,O as T,Q as jt,bh as oe,c$ as we,V as ie,e4 as se,cu as Ot,aL as It,e7 as Ht,m as Ft,a1 as Dt,db as Vt,ci as le,br as Q,e2 as Mt,cE as Nt,dn as O,R as Z}from"./index-Bu37ZZhq.js";import{u as fe}from"./Tooltip-dDog9Bhu.js";import{A as Xt}from"./Add-CLyE3lrC.js";import{e as Ut,d as Ce,o as Yt}from"./FocusDetector-DKHtM3pi.js";import{a as Gt}from"./Input-CLNG5k6Y.js";const Kt=Ce(".v-x-scroll",{overflow:"auto",scrollbarWidth:"none"},[Ce("&::-webkit-scrollbar",{width:0,height:0})]),qt=G({name:"XScroll",props:{disabled:Boolean,onScroll:Function},setup(){const e=L(null);function r(d){!(d.currentTarget.offsetWidth<d.currentTarget.scrollWidth)||d.deltaY===0||(d.currentTarget.scrollLeft+=d.deltaY+d.deltaX,d.preventDefault())}const i=xt();return Kt.mount({id:"vueuc/x-scroll",head:!0,anchorMetaName:Ut,ssr:i}),Object.assign({selfRef:e,handleWheel:r},{scrollTo(...d){var h;(h=e.value)===null||h===void 0||h.scrollTo(...d)}})},render(){return l("div",{ref:"selfRef",onScroll:this.onScroll,onWheel:this.disabled?void 0:this.handleWheel,class:"v-x-scroll"},this.$slots)}});var Jt=/\s/;function Qt(e){for(var r=e.length;r--&&Jt.test(e.charAt(r)););return r}var Zt=/^\s+/;function ea(e){return e&&e.slice(0,Qt(e)+1).replace(Zt,"")}var Re=NaN,ta=/^[-+]0x[0-9a-f]+$/i,aa=/^0b[01]+$/i,na=/^0o[0-7]+$/i,ra=parseInt;function ze(e){if(typeof e=="number")return e;if(yt(e))return Re;if(ee(e)){var r=typeof e.valueOf=="function"?e.valueOf():e;e=ee(r)?r+"":r}if(typeof e!="string")return e===0?e:+e;e=ea(e);var i=aa.test(e);return i||na.test(e)?ra(e.slice(2),i?2:8):ta.test(e)?Re:+e}var de=function(){return St.Date.now()},oa="Expected a function",ia=Math.max,sa=Math.min;function la(e,r,i){var c,d,h,v,u,p,g=0,x=!1,R=!1,$=!0;if(typeof e!="function")throw new TypeError(oa);r=ze(r)||0,ee(i)&&(x=!!i.leading,R="maxWait"in i,h=R?ia(ze(i.maxWait)||0,r):h,$="trailing"in i?!!i.trailing:$);function w(b){var E=c,D=d;return c=d=void 0,g=b,v=e.apply(D,E),v}function S(b){return g=b,u=setTimeout(W,r),x?w(b):v}function z(b){var E=b-p,D=b-g,V=r-E;return R?sa(V,h-D):V}function _(b){var E=b-p,D=b-g;return p===void 0||E>=r||E<0||R&&D>=h}function W(){var b=de();if(_(b))return P(b);u=setTimeout(W,z(b))}function P(b){return u=void 0,$&&c?w(b):(c=d=void 0,v)}function I(){u!==void 0&&clearTimeout(u),g=0,c=p=d=u=void 0}function j(){return u===void 0?v:P(de())}function m(){var b=de(),E=_(b);if(c=arguments,d=this,p=b,E){if(u===void 0)return S(p);if(R)return clearTimeout(u),u=setTimeout(W,r),w(p)}return u===void 0&&(u=setTimeout(W,r)),v}return m.cancel=I,m.flush=j,m}var da="Expected a function";function ca(e,r,i){var c=!0,d=!0;if(typeof e!="function")throw new TypeError(da);return ee(i)&&(c="leading"in i?!!i.leading:c,d="trailing"in i?!!i.trailing:d),la(e,r,{leading:c,maxWait:r,trailing:d})}const ba=y([y("@keyframes spin-rotate",`
 from {
 transform: rotate(0);
 }
 to {
 transform: rotate(360deg);
 }
 `),n("spin-container",`
 position: relative;
 `,[n("spin-body",`
 position: absolute;
 top: 50%;
 left: 50%;
 transform: translateX(-50%) translateY(-50%);
 `,[wt()])]),n("spin-body",`
 display: inline-flex;
 align-items: center;
 justify-content: center;
 flex-direction: column;
 `),n("spin",`
 display: inline-flex;
 height: var(--n-size);
 width: var(--n-size);
 font-size: var(--n-size);
 color: var(--n-color);
 `,[s("rotate",`
 animation: spin-rotate 2s linear infinite;
 `)]),n("spin-description",`
 display: inline-block;
 font-size: var(--n-font-size);
 color: var(--n-text-color);
 transition: color .3s var(--n-bezier);
 margin-top: 8px;
 `),n("spin-content",`
 opacity: 1;
 transition: opacity .3s var(--n-bezier);
 pointer-events: all;
 `,[s("spinning",`
 user-select: none;
 -webkit-user-select: none;
 pointer-events: none;
 opacity: var(--n-opacity-spinning);
 `)])]),fa={small:20,medium:18,large:16},pa=Object.assign(Object.assign(Object.assign({},te.props),{contentClass:String,contentStyle:[Object,String],description:String,size:{type:[String,Number],default:"medium"},show:{type:Boolean,default:!0},rotate:{type:Boolean,default:!0},spinning:{type:Boolean,validator:()=>!0,default:void 0},delay:Number}),Tt),wa=G({name:"Spin",props:pa,slots:Object,setup(e){const{mergedClsPrefixRef:r,inlineThemeDisabled:i}=_e(e),c=te("Spin","-spin",ba,zt,e,r),d=F(()=>{const{size:p}=e,{common:{cubicBezierEaseInOut:g},self:x}=c.value,{opacitySpinning:R,color:$,textColor:w}=x,S=typeof p=="number"?$t(p):x[A("size",p)];return{"--n-bezier":g,"--n-opacity-spinning":R,"--n-size":S,"--n-color":$,"--n-text-color":w}}),h=i?We("spin",F(()=>{const{size:p}=e;return typeof p=="number"?String(p):p[0]}),d,e):void 0,v=fe(e,["spinning","show"]),u=L(!1);return Ee(p=>{let g;if(v.value){const{delay:x}=e;if(x){g=window.setTimeout(()=>{u.value=!0},x),p(()=>{clearTimeout(g)});return}}u.value=v.value}),{mergedClsPrefix:r,active:u,mergedStrokeWidth:F(()=>{const{strokeWidth:p}=e;if(p!==void 0)return p;const{size:g}=e;return fa[typeof g=="number"?"medium":g]}),cssVars:i?void 0:d,themeClass:h==null?void 0:h.themeClass,onRender:h==null?void 0:h.onRender}},render(){var e,r;const{$slots:i,mergedClsPrefix:c,description:d}=this,h=i.icon&&this.rotate,v=(d||i.description)&&l("div",{class:`${c}-spin-description`},d||((e=i.description)===null||e===void 0?void 0:e.call(i))),u=i.icon?l("div",{class:[`${c}-spin-body`,this.themeClass]},l("div",{class:[`${c}-spin`,h&&`${c}-spin--rotate`],style:i.default?"":this.cssVars},i.icon()),v):l("div",{class:[`${c}-spin-body`,this.themeClass]},l(Ct,{clsPrefix:c,style:i.default?"":this.cssVars,stroke:this.stroke,"stroke-width":this.mergedStrokeWidth,radius:this.radius,scale:this.scale,class:`${c}-spin`}),v);return(r=this.onRender)===null||r===void 0||r.call(this),i.default?l("div",{class:[`${c}-spin-container`,this.themeClass],style:this.cssVars},l("div",{class:[`${c}-spin-content`,this.active&&`${c}-spin-content--spinning`,this.contentClass],style:this.contentStyle},i),l(Rt,{name:"fade-in-transition"},{default:()=>this.active?u:null})):u}}),ue=Pt("n-tabs"),ke={tab:[String,Number,Object,Function],name:{type:[String,Number],required:!0},disabled:Boolean,displayDirective:{type:String,default:"if"},closable:{type:Boolean,default:void 0},tabProps:Object,label:[String,Number,Object,Function]},Ca=G({__TAB_PANE__:!0,name:"TabPane",alias:["TabPanel"],props:ke,slots:Object,setup(e){const r=Le(ue,null);return r||_t("tab-pane","`n-tab-pane` must be placed inside `n-tabs`."),{style:r.paneStyleRef,class:r.paneClassRef,mergedClsPrefix:r.mergedClsPrefixRef}},render(){return l("div",{class:[`${this.mergedClsPrefix}-tab-pane`,this.class],style:this.style},this.$slots)}}),ua=Object.assign({internalLeftPadded:Boolean,internalAddable:Boolean,internalCreatedByPane:Boolean},At(ke,["displayDirective"])),pe=G({__TAB__:!0,inheritAttrs:!1,name:"Tab",props:ua,setup(e){const{mergedClsPrefixRef:r,valueRef:i,typeRef:c,closableRef:d,tabStyleRef:h,addTabStyleRef:v,tabClassRef:u,addTabClassRef:p,tabChangeIdRef:g,onBeforeLeaveRef:x,triggerRef:R,handleAdd:$,activateTab:w,handleClose:S}=Le(ue);return{trigger:R,mergedClosable:F(()=>{if(e.internalAddable)return!1;const{closable:z}=e;return z===void 0?d.value:z}),style:h,addStyle:v,tabClass:u,addTabClass:p,clsPrefix:r,value:i,type:c,handleClose(z){z.stopPropagation(),!e.disabled&&S(e.name)},activateTab(){if(e.disabled)return;if(e.internalAddable){$();return}const{name:z}=e,_=++g.id;if(z!==i.value){const{value:W}=x;W?Promise.resolve(W(e.name,i.value)).then(P=>{P&&g.id===_&&w(z)}):w(z)}}}},render(){const{internalAddable:e,clsPrefix:r,name:i,disabled:c,label:d,tab:h,value:v,mergedClosable:u,trigger:p,$slots:{default:g}}=this,x=d??h;return l("div",{class:`${r}-tabs-tab-wrapper`},this.internalLeftPadded?l("div",{class:`${r}-tabs-tab-pad`}):null,l("div",Object.assign({key:i,"data-name":i,"data-disabled":c?!0:void 0},Wt({class:[`${r}-tabs-tab`,v===i&&`${r}-tabs-tab--active`,c&&`${r}-tabs-tab--disabled`,u&&`${r}-tabs-tab--closable`,e&&`${r}-tabs-tab--addable`,e?this.addTabClass:this.tabClass],onClick:p==="click"?this.activateTab:void 0,onMouseenter:p==="hover"?this.activateTab:void 0,style:e?this.addStyle:this.style},this.internalCreatedByPane?this.tabProps||{}:this.$attrs)),l("span",{class:`${r}-tabs-tab__label`},e?l(Et,null,l("div",{class:`${r}-tabs-tab__height-placeholder`}," "),l(Lt,{clsPrefix:r},{default:()=>l(Xt,null)})):g?g():typeof x=="object"?x:kt(x??i)),u&&this.type==="card"?l(Bt,{clsPrefix:r,class:`${r}-tabs-tab__close`,onClick:this.handleClose,disabled:c}):null))}}),va=n("tabs",`
 box-sizing: border-box;
 width: 100%;
 display: flex;
 flex-direction: column;
 transition:
 background-color .3s var(--n-bezier),
 border-color .3s var(--n-bezier);
`,[s("segment-type",[n("tabs-rail",[y("&.transition-disabled",[n("tabs-capsule",`
 transition: none;
 `)])])]),s("top",[n("tab-pane",`
 padding: var(--n-pane-padding-top) var(--n-pane-padding-right) var(--n-pane-padding-bottom) var(--n-pane-padding-left);
 `)]),s("left",[n("tab-pane",`
 padding: var(--n-pane-padding-right) var(--n-pane-padding-bottom) var(--n-pane-padding-left) var(--n-pane-padding-top);
 `)]),s("left, right",`
 flex-direction: row;
 `,[n("tabs-bar",`
 width: 2px;
 right: 0;
 transition:
 top .2s var(--n-bezier),
 max-height .2s var(--n-bezier),
 background-color .3s var(--n-bezier);
 `),n("tabs-tab",`
 padding: var(--n-tab-padding-vertical); 
 `)]),s("right",`
 flex-direction: row-reverse;
 `,[n("tab-pane",`
 padding: var(--n-pane-padding-left) var(--n-pane-padding-top) var(--n-pane-padding-right) var(--n-pane-padding-bottom);
 `),n("tabs-bar",`
 left: 0;
 `)]),s("bottom",`
 flex-direction: column-reverse;
 justify-content: flex-end;
 `,[n("tab-pane",`
 padding: var(--n-pane-padding-bottom) var(--n-pane-padding-right) var(--n-pane-padding-top) var(--n-pane-padding-left);
 `),n("tabs-bar",`
 top: 0;
 `)]),n("tabs-rail",`
 position: relative;
 padding: 3px;
 border-radius: var(--n-tab-border-radius);
 width: 100%;
 background-color: var(--n-color-segment);
 transition: background-color .3s var(--n-bezier);
 display: flex;
 align-items: center;
 `,[n("tabs-capsule",`
 border-radius: var(--n-tab-border-radius);
 position: absolute;
 pointer-events: none;
 background-color: var(--n-tab-color-segment);
 box-shadow: 0 1px 3px 0 rgba(0, 0, 0, .08);
 transition: transform 0.3s var(--n-bezier);
 `),n("tabs-tab-wrapper",`
 flex-basis: 0;
 flex-grow: 1;
 display: flex;
 align-items: center;
 justify-content: center;
 `,[n("tabs-tab",`
 overflow: hidden;
 border-radius: var(--n-tab-border-radius);
 width: 100%;
 display: flex;
 align-items: center;
 justify-content: center;
 `,[s("active",`
 font-weight: var(--n-font-weight-strong);
 color: var(--n-tab-text-color-active);
 `),y("&:hover",`
 color: var(--n-tab-text-color-hover);
 `)])])]),s("flex",[n("tabs-nav",`
 width: 100%;
 position: relative;
 `,[n("tabs-wrapper",`
 width: 100%;
 `,[n("tabs-tab",`
 margin-right: 0;
 `)])])]),n("tabs-nav",`
 box-sizing: border-box;
 line-height: 1.5;
 display: flex;
 transition: border-color .3s var(--n-bezier);
 `,[T("prefix, suffix",`
 display: flex;
 align-items: center;
 `),T("prefix","padding-right: 16px;"),T("suffix","padding-left: 16px;")]),s("top, bottom",[y(">",[n("tabs-nav",[n("tabs-nav-scroll-wrapper",[y("&::before",`
 top: 0;
 bottom: 0;
 left: 0;
 width: 20px;
 `),y("&::after",`
 top: 0;
 bottom: 0;
 right: 0;
 width: 20px;
 `),s("shadow-start",[y("&::before",`
 box-shadow: inset 10px 0 8px -8px rgba(0, 0, 0, .12);
 `)]),s("shadow-end",[y("&::after",`
 box-shadow: inset -10px 0 8px -8px rgba(0, 0, 0, .12);
 `)])])])])]),s("left, right",[n("tabs-nav-scroll-content",`
 flex-direction: column;
 `),y(">",[n("tabs-nav",[n("tabs-nav-scroll-wrapper",[y("&::before",`
 top: 0;
 left: 0;
 right: 0;
 height: 20px;
 `),y("&::after",`
 bottom: 0;
 left: 0;
 right: 0;
 height: 20px;
 `),s("shadow-start",[y("&::before",`
 box-shadow: inset 0 10px 8px -8px rgba(0, 0, 0, .12);
 `)]),s("shadow-end",[y("&::after",`
 box-shadow: inset 0 -10px 8px -8px rgba(0, 0, 0, .12);
 `)])])])])]),n("tabs-nav-scroll-wrapper",`
 flex: 1;
 position: relative;
 overflow: hidden;
 `,[n("tabs-nav-y-scroll",`
 height: 100%;
 width: 100%;
 overflow-y: auto; 
 scrollbar-width: none;
 `,[y("&::-webkit-scrollbar, &::-webkit-scrollbar-track-piece, &::-webkit-scrollbar-thumb",`
 width: 0;
 height: 0;
 display: none;
 `)]),y("&::before, &::after",`
 transition: box-shadow .3s var(--n-bezier);
 pointer-events: none;
 content: "";
 position: absolute;
 z-index: 1;
 `)]),n("tabs-nav-scroll-content",`
 display: flex;
 position: relative;
 min-width: 100%;
 min-height: 100%;
 width: fit-content;
 box-sizing: border-box;
 `),n("tabs-wrapper",`
 display: inline-flex;
 flex-wrap: nowrap;
 position: relative;
 `),n("tabs-tab-wrapper",`
 display: flex;
 flex-wrap: nowrap;
 flex-shrink: 0;
 flex-grow: 0;
 `),n("tabs-tab",`
 cursor: pointer;
 white-space: nowrap;
 flex-wrap: nowrap;
 display: inline-flex;
 align-items: center;
 color: var(--n-tab-text-color);
 font-size: var(--n-tab-font-size);
 background-clip: padding-box;
 padding: var(--n-tab-padding);
 transition:
 box-shadow .3s var(--n-bezier),
 color .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 border-color .3s var(--n-bezier);
 `,[s("disabled",{cursor:"not-allowed"}),T("close",`
 margin-left: 6px;
 transition:
 background-color .3s var(--n-bezier),
 color .3s var(--n-bezier);
 `),T("label",`
 display: flex;
 align-items: center;
 z-index: 1;
 `)]),n("tabs-bar",`
 position: absolute;
 bottom: 0;
 height: 2px;
 border-radius: 1px;
 background-color: var(--n-bar-color);
 transition:
 left .2s var(--n-bezier),
 max-width .2s var(--n-bezier),
 opacity .3s var(--n-bezier),
 background-color .3s var(--n-bezier);
 `,[y("&.transition-disabled",`
 transition: none;
 `),s("disabled",`
 background-color: var(--n-tab-text-color-disabled)
 `)]),n("tabs-pane-wrapper",`
 position: relative;
 overflow: hidden;
 transition: max-height .2s var(--n-bezier);
 `),n("tab-pane",`
 color: var(--n-pane-text-color);
 width: 100%;
 transition:
 color .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 opacity .2s var(--n-bezier);
 left: 0;
 right: 0;
 top: 0;
 `,[y("&.next-transition-leave-active, &.prev-transition-leave-active, &.next-transition-enter-active, &.prev-transition-enter-active",`
 transition:
 color .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 transform .2s var(--n-bezier),
 opacity .2s var(--n-bezier);
 `),y("&.next-transition-leave-active, &.prev-transition-leave-active",`
 position: absolute;
 `),y("&.next-transition-enter-from, &.prev-transition-leave-to",`
 transform: translateX(32px);
 opacity: 0;
 `),y("&.next-transition-leave-to, &.prev-transition-enter-from",`
 transform: translateX(-32px);
 opacity: 0;
 `),y("&.next-transition-leave-from, &.next-transition-enter-to, &.prev-transition-leave-from, &.prev-transition-enter-to",`
 transform: translateX(0);
 opacity: 1;
 `)]),n("tabs-tab-pad",`
 box-sizing: border-box;
 width: var(--n-tab-gap);
 flex-grow: 0;
 flex-shrink: 0;
 `),s("line-type, bar-type",[n("tabs-tab",`
 font-weight: var(--n-tab-font-weight);
 box-sizing: border-box;
 vertical-align: bottom;
 `,[y("&:hover",{color:"var(--n-tab-text-color-hover)"}),s("active",`
 color: var(--n-tab-text-color-active);
 font-weight: var(--n-tab-font-weight-active);
 `),s("disabled",{color:"var(--n-tab-text-color-disabled)"})])]),n("tabs-nav",[s("line-type",[s("top",[T("prefix, suffix",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `),n("tabs-nav-scroll-content",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `),n("tabs-bar",`
 bottom: -1px;
 `)]),s("left",[T("prefix, suffix",`
 border-right: 1px solid var(--n-tab-border-color);
 `),n("tabs-nav-scroll-content",`
 border-right: 1px solid var(--n-tab-border-color);
 `),n("tabs-bar",`
 right: -1px;
 `)]),s("right",[T("prefix, suffix",`
 border-left: 1px solid var(--n-tab-border-color);
 `),n("tabs-nav-scroll-content",`
 border-left: 1px solid var(--n-tab-border-color);
 `),n("tabs-bar",`
 left: -1px;
 `)]),s("bottom",[T("prefix, suffix",`
 border-top: 1px solid var(--n-tab-border-color);
 `),n("tabs-nav-scroll-content",`
 border-top: 1px solid var(--n-tab-border-color);
 `),n("tabs-bar",`
 top: -1px;
 `)]),T("prefix, suffix",`
 transition: border-color .3s var(--n-bezier);
 `),n("tabs-nav-scroll-content",`
 transition: border-color .3s var(--n-bezier);
 `),n("tabs-bar",`
 border-radius: 0;
 `)]),s("card-type",[T("prefix, suffix",`
 transition: border-color .3s var(--n-bezier);
 `),n("tabs-pad",`
 flex-grow: 1;
 transition: border-color .3s var(--n-bezier);
 `),n("tabs-tab-pad",`
 transition: border-color .3s var(--n-bezier);
 `),n("tabs-tab",`
 font-weight: var(--n-tab-font-weight);
 border: 1px solid var(--n-tab-border-color);
 background-color: var(--n-tab-color);
 box-sizing: border-box;
 position: relative;
 vertical-align: bottom;
 display: flex;
 justify-content: space-between;
 font-size: var(--n-tab-font-size);
 color: var(--n-tab-text-color);
 `,[s("addable",`
 padding-left: 8px;
 padding-right: 8px;
 font-size: 16px;
 justify-content: center;
 `,[T("height-placeholder",`
 width: 0;
 font-size: var(--n-tab-font-size);
 `),jt("disabled",[y("&:hover",`
 color: var(--n-tab-text-color-hover);
 `)])]),s("closable","padding-right: 8px;"),s("active",`
 background-color: #0000;
 font-weight: var(--n-tab-font-weight-active);
 color: var(--n-tab-text-color-active);
 `),s("disabled","color: var(--n-tab-text-color-disabled);")])]),s("left, right",`
 flex-direction: column; 
 `,[T("prefix, suffix",`
 padding: var(--n-tab-padding-vertical);
 `),n("tabs-wrapper",`
 flex-direction: column;
 `),n("tabs-tab-wrapper",`
 flex-direction: column;
 `,[n("tabs-tab-pad",`
 height: var(--n-tab-gap-vertical);
 width: 100%;
 `)])]),s("top",[s("card-type",[n("tabs-scroll-padding","border-bottom: 1px solid var(--n-tab-border-color);"),T("prefix, suffix",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `),n("tabs-tab",`
 border-top-left-radius: var(--n-tab-border-radius);
 border-top-right-radius: var(--n-tab-border-radius);
 `,[s("active",`
 border-bottom: 1px solid #0000;
 `)]),n("tabs-tab-pad",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `),n("tabs-pad",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `)])]),s("left",[s("card-type",[n("tabs-scroll-padding","border-right: 1px solid var(--n-tab-border-color);"),T("prefix, suffix",`
 border-right: 1px solid var(--n-tab-border-color);
 `),n("tabs-tab",`
 border-top-left-radius: var(--n-tab-border-radius);
 border-bottom-left-radius: var(--n-tab-border-radius);
 `,[s("active",`
 border-right: 1px solid #0000;
 `)]),n("tabs-tab-pad",`
 border-right: 1px solid var(--n-tab-border-color);
 `),n("tabs-pad",`
 border-right: 1px solid var(--n-tab-border-color);
 `)])]),s("right",[s("card-type",[n("tabs-scroll-padding","border-left: 1px solid var(--n-tab-border-color);"),T("prefix, suffix",`
 border-left: 1px solid var(--n-tab-border-color);
 `),n("tabs-tab",`
 border-top-right-radius: var(--n-tab-border-radius);
 border-bottom-right-radius: var(--n-tab-border-radius);
 `,[s("active",`
 border-left: 1px solid #0000;
 `)]),n("tabs-tab-pad",`
 border-left: 1px solid var(--n-tab-border-color);
 `),n("tabs-pad",`
 border-left: 1px solid var(--n-tab-border-color);
 `)])]),s("bottom",[s("card-type",[n("tabs-scroll-padding","border-top: 1px solid var(--n-tab-border-color);"),T("prefix, suffix",`
 border-top: 1px solid var(--n-tab-border-color);
 `),n("tabs-tab",`
 border-bottom-left-radius: var(--n-tab-border-radius);
 border-bottom-right-radius: var(--n-tab-border-radius);
 `,[s("active",`
 border-top: 1px solid #0000;
 `)]),n("tabs-tab-pad",`
 border-top: 1px solid var(--n-tab-border-color);
 `),n("tabs-pad",`
 border-top: 1px solid var(--n-tab-border-color);
 `)])])])]),ce=ca,ha=Object.assign(Object.assign({},te.props),{value:[String,Number],defaultValue:[String,Number],trigger:{type:String,default:"click"},type:{type:String,default:"bar"},closable:Boolean,justifyContent:String,size:String,placement:{type:String,default:"top"},tabStyle:[String,Object],tabClass:String,addTabStyle:[String,Object],addTabClass:String,barWidth:Number,paneClass:String,paneStyle:[String,Object],paneWrapperClass:String,paneWrapperStyle:[String,Object],addable:[Boolean,Object],tabsPadding:{type:Number,default:0},animated:Boolean,onBeforeLeave:Function,onAdd:Function,"onUpdate:value":[Function,Array],onUpdateValue:[Function,Array],onClose:[Function,Array],labelSize:String,activeName:[String,Number],onActiveNameChange:[Function,Array]}),Ra=G({name:"Tabs",props:ha,slots:Object,setup(e,{slots:r}){var i,c,d,h;const{mergedClsPrefixRef:v,inlineThemeDisabled:u,mergedComponentPropsRef:p}=_e(e),g=te("Tabs","-tabs",va,Vt,e,v),x=L(null),R=L(null),$=L(null),w=L(null),S=L(null),z=L(null),_=L(!0),W=L(!0),P=fe(e,["labelSize","size"]),I=F(()=>{var t,a;if(P.value)return P.value;const o=(a=(t=p==null?void 0:p.value)===null||t===void 0?void 0:t.Tabs)===null||a===void 0?void 0:a.size;return o||"medium"}),j=fe(e,["activeName","value"]),m=L((c=(i=j.value)!==null&&i!==void 0?i:e.defaultValue)!==null&&c!==void 0?c:r.default?(h=(d=oe(r.default())[0])===null||d===void 0?void 0:d.props)===null||h===void 0?void 0:h.name:null),b=Gt(j,m),E={id:0},D=F(()=>{if(!(!e.justifyContent||e.type==="card"))return{display:"flex",justifyContent:e.justifyContent}});se(b,()=>{E.id=0,K(),he()});function V(){var t;const{value:a}=b;return a===null?null:(t=x.value)===null||t===void 0?void 0:t.querySelector(`[data-name="${a}"]`)}function Be(t){if(e.type==="card")return;const{value:a}=R;if(!a)return;const o=a.style.opacity==="0";if(t){const f=`${v.value}-tabs-bar--disabled`,{barWidth:C,placement:k}=e;if(t.dataset.disabled==="true"?a.classList.add(f):a.classList.remove(f),["top","bottom"].includes(k)){if(ve(["top","maxHeight","height"]),typeof C=="number"&&t.offsetWidth>=C){const B=Math.floor((t.offsetWidth-C)/2)+t.offsetLeft;a.style.left=`${B}px`,a.style.maxWidth=`${C}px`}else a.style.left=`${t.offsetLeft}px`,a.style.maxWidth=`${t.offsetWidth}px`;a.style.width="8192px",o&&(a.style.transition="none"),a.offsetWidth,o&&(a.style.transition="",a.style.opacity="1")}else{if(ve(["left","maxWidth","width"]),typeof C=="number"&&t.offsetHeight>=C){const B=Math.floor((t.offsetHeight-C)/2)+t.offsetTop;a.style.top=`${B}px`,a.style.maxHeight=`${C}px`}else a.style.top=`${t.offsetTop}px`,a.style.maxHeight=`${t.offsetHeight}px`;a.style.height="8192px",o&&(a.style.transition="none"),a.offsetHeight,o&&(a.style.transition="",a.style.opacity="1")}}}function Ae(){if(e.type==="card")return;const{value:t}=R;t&&(t.style.opacity="0")}function ve(t){const{value:a}=R;if(a)for(const o of t)a.style[o]=""}function K(){if(e.type==="card")return;const t=V();t?Be(t):Ae()}function he(){var t;const a=(t=S.value)===null||t===void 0?void 0:t.$el;if(!a)return;const o=V();if(!o)return;const{scrollLeft:f,offsetWidth:C}=a,{offsetLeft:k,offsetWidth:B}=o;f>k?a.scrollTo({top:0,left:k,behavior:"smooth"}):k+B>f+C&&a.scrollTo({top:0,left:k+B-C,behavior:"smooth"})}const q=L(null);let ae=0,H=null;function je(t){const a=q.value;if(a){ae=t.getBoundingClientRect().height;const o=`${ae}px`,f=()=>{a.style.height=o,a.style.maxHeight=o};H?(f(),H(),H=null):H=f}}function Oe(t){const a=q.value;if(a){const o=t.getBoundingClientRect().height,f=()=>{document.body.offsetHeight,a.style.maxHeight=`${o}px`,a.style.height=`${Math.max(ae,o)}px`};H?(H(),H=null,f()):H=f}}function Ie(){const t=q.value;if(t){t.style.maxHeight="",t.style.height="";const{paneWrapperStyle:a}=e;if(typeof a=="string")t.style.cssText=a;else if(a){const{maxHeight:o,height:f}=a;o!==void 0&&(t.style.maxHeight=o),f!==void 0&&(t.style.height=f)}}}const ge={value:[]},me=L("next");function He(t){const a=b.value;let o="next";for(const f of ge.value){if(f===a)break;if(f===t){o="prev";break}}me.value=o,Fe(t)}function Fe(t){const{onActiveNameChange:a,onUpdateValue:o,"onUpdate:value":f}=e;a&&Z(a,t),o&&Z(o,t),f&&Z(f,t),m.value=t}function De(t){const{onClose:a}=e;a&&Z(a,t)}function xe(){const{value:t}=R;if(!t)return;const a="transition-disabled";t.classList.add(a),K(),t.classList.remove(a)}const M=L(null);function ne({transitionDisabled:t}){const a=x.value;if(!a)return;t&&a.classList.add("transition-disabled");const o=V();o&&M.value&&(M.value.style.width=`${o.offsetWidth}px`,M.value.style.height=`${o.offsetHeight}px`,M.value.style.transform=`translateX(${o.offsetLeft-It(getComputedStyle(a).paddingLeft)}px)`,t&&M.value.offsetWidth),t&&a.classList.remove("transition-disabled")}se([b],()=>{e.type==="segment"&&le(()=>{ne({transitionDisabled:!1})})}),Ot(()=>{e.type==="segment"&&ne({transitionDisabled:!0})});let ye=0;function Ve(t){var a;if(t.contentRect.width===0&&t.contentRect.height===0||ye===t.contentRect.width)return;ye=t.contentRect.width;const{type:o}=e;if((o==="line"||o==="bar")&&xe(),o!=="segment"){const{placement:f}=e;re((f==="top"||f==="bottom"?(a=S.value)===null||a===void 0?void 0:a.$el:z.value)||null)}}const Me=ce(Ve,64);se([()=>e.justifyContent,()=>e.size],()=>{le(()=>{const{type:t}=e;(t==="line"||t==="bar")&&xe()})});const N=L(!1);function Ne(t){var a;const{target:o,contentRect:{width:f,height:C}}=t,k=o.parentElement.parentElement.offsetWidth,B=o.parentElement.parentElement.offsetHeight,{placement:U}=e;if(!N.value)U==="top"||U==="bottom"?k<f&&(N.value=!0):B<C&&(N.value=!0);else{const{value:Y}=w;if(!Y)return;U==="top"||U==="bottom"?k-f>Y.$el.offsetWidth&&(N.value=!1):B-C>Y.$el.offsetHeight&&(N.value=!1)}re(((a=S.value)===null||a===void 0?void 0:a.$el)||null)}const Xe=ce(Ne,64);function Ue(){const{onAdd:t}=e;t&&t(),le(()=>{const a=V(),{value:o}=S;!a||!o||o.scrollTo({left:a.offsetLeft,top:0,behavior:"smooth"})})}function re(t){if(!t)return;const{placement:a}=e;if(a==="top"||a==="bottom"){const{scrollLeft:o,scrollWidth:f,offsetWidth:C}=t;_.value=o<=0,W.value=o+C>=f}else{const{scrollTop:o,scrollHeight:f,offsetHeight:C}=t;_.value=o<=0,W.value=o+C>=f}}const Ye=ce(t=>{re(t.target)},64);Nt(ue,{triggerRef:O(e,"trigger"),tabStyleRef:O(e,"tabStyle"),tabClassRef:O(e,"tabClass"),addTabStyleRef:O(e,"addTabStyle"),addTabClassRef:O(e,"addTabClass"),paneClassRef:O(e,"paneClass"),paneStyleRef:O(e,"paneStyle"),mergedClsPrefixRef:v,typeRef:O(e,"type"),closableRef:O(e,"closable"),valueRef:b,tabChangeIdRef:E,onBeforeLeaveRef:O(e,"onBeforeLeave"),activateTab:He,handleClose:De,handleAdd:Ue}),Yt(()=>{K(),he()}),Ee(()=>{const{value:t}=$;if(!t)return;const{value:a}=v,o=`${a}-tabs-nav-scroll-wrapper--shadow-start`,f=`${a}-tabs-nav-scroll-wrapper--shadow-end`;_.value?t.classList.remove(o):t.classList.add(o),W.value?t.classList.remove(f):t.classList.add(f)});const Ge={syncBarPosition:()=>{K()}},Ke=()=>{ne({transitionDisabled:!0})},Se=F(()=>{const{value:t}=I,{type:a}=e,o={card:"Card",bar:"Bar",line:"Line",segment:"Segment"}[a],f=`${t}${o}`,{self:{barColor:C,closeIconColor:k,closeIconColorHover:B,closeIconColorPressed:U,tabColor:Y,tabBorderColor:qe,paneTextColor:Je,tabFontWeight:Qe,tabBorderRadius:Ze,tabFontWeightActive:et,colorSegment:tt,fontWeightStrong:at,tabColorSegment:nt,closeSize:rt,closeIconSize:ot,closeColorHover:it,closeColorPressed:st,closeBorderRadius:lt,[A("panePadding",t)]:J,[A("tabPadding",f)]:dt,[A("tabPaddingVertical",f)]:ct,[A("tabGap",f)]:bt,[A("tabGap",`${f}Vertical`)]:ft,[A("tabTextColor",a)]:pt,[A("tabTextColorActive",a)]:ut,[A("tabTextColorHover",a)]:vt,[A("tabTextColorDisabled",a)]:ht,[A("tabFontSize",t)]:gt},common:{cubicBezierEaseInOut:mt}}=g.value;return{"--n-bezier":mt,"--n-color-segment":tt,"--n-bar-color":C,"--n-tab-font-size":gt,"--n-tab-text-color":pt,"--n-tab-text-color-active":ut,"--n-tab-text-color-disabled":ht,"--n-tab-text-color-hover":vt,"--n-pane-text-color":Je,"--n-tab-border-color":qe,"--n-tab-border-radius":Ze,"--n-close-size":rt,"--n-close-icon-size":ot,"--n-close-color-hover":it,"--n-close-color-pressed":st,"--n-close-border-radius":lt,"--n-close-icon-color":k,"--n-close-icon-color-hover":B,"--n-close-icon-color-pressed":U,"--n-tab-color":Y,"--n-tab-font-weight":Qe,"--n-tab-font-weight-active":et,"--n-tab-padding":dt,"--n-tab-padding-vertical":ct,"--n-tab-gap":bt,"--n-tab-gap-vertical":ft,"--n-pane-padding-left":Q(J,"left"),"--n-pane-padding-right":Q(J,"right"),"--n-pane-padding-top":Q(J,"top"),"--n-pane-padding-bottom":Q(J,"bottom"),"--n-font-weight-strong":at,"--n-tab-color-segment":nt}}),X=u?We("tabs",F(()=>`${I.value[0]}${e.type[0]}`),Se,e):void 0;return Object.assign({mergedClsPrefix:v,mergedValue:b,renderedNames:new Set,segmentCapsuleElRef:M,tabsPaneWrapperRef:q,tabsElRef:x,barElRef:R,addTabInstRef:w,xScrollInstRef:S,scrollWrapperElRef:$,addTabFixed:N,tabWrapperStyle:D,handleNavResize:Me,mergedSize:I,handleScroll:Ye,handleTabsResize:Xe,cssVars:u?void 0:Se,themeClass:X==null?void 0:X.themeClass,animationDirection:me,renderNameListRef:ge,yScrollElRef:z,handleSegmentResize:Ke,onAnimationBeforeLeave:je,onAnimationEnter:Oe,onAnimationAfterEnter:Ie,onRender:X==null?void 0:X.onRender},Ge)},render(){const{mergedClsPrefix:e,type:r,placement:i,addTabFixed:c,addable:d,mergedSize:h,renderNameListRef:v,onRender:u,paneWrapperClass:p,paneWrapperStyle:g,$slots:{default:x,prefix:R,suffix:$}}=this;u==null||u();const w=x?oe(x()).filter(m=>m.type.__TAB_PANE__===!0):[],S=x?oe(x()).filter(m=>m.type.__TAB__===!0):[],z=!S.length,_=r==="card",W=r==="segment",P=!_&&!W&&this.justifyContent;v.value=[];const I=()=>{const m=l("div",{style:this.tabWrapperStyle,class:`${e}-tabs-wrapper`},P?null:l("div",{class:`${e}-tabs-scroll-padding`,style:i==="top"||i==="bottom"?{width:`${this.tabsPadding}px`}:{height:`${this.tabsPadding}px`}}),z?w.map((b,E)=>(v.value.push(b.props.name),be(l(pe,Object.assign({},b.props,{internalCreatedByPane:!0,internalLeftPadded:E!==0&&(!P||P==="center"||P==="start"||P==="end")}),b.children?{default:b.children.tab}:void 0)))):S.map((b,E)=>(v.value.push(b.props.name),be(E!==0&&!P?Pe(b):b))),!c&&d&&_?$e(d,(z?w.length:S.length)!==0):null,P?null:l("div",{class:`${e}-tabs-scroll-padding`,style:{width:`${this.tabsPadding}px`}}));return l("div",{ref:"tabsElRef",class:`${e}-tabs-nav-scroll-content`},_&&d?l(ie,{onResize:this.handleTabsResize},{default:()=>m}):m,_?l("div",{class:`${e}-tabs-pad`}):null,_?null:l("div",{ref:"barElRef",class:`${e}-tabs-bar`}))},j=W?"top":i;return l("div",{class:[`${e}-tabs`,this.themeClass,`${e}-tabs--${r}-type`,`${e}-tabs--${h}-size`,P&&`${e}-tabs--flex`,`${e}-tabs--${j}`],style:this.cssVars},l("div",{class:[`${e}-tabs-nav--${r}-type`,`${e}-tabs-nav--${j}`,`${e}-tabs-nav`]},we(R,m=>m&&l("div",{class:`${e}-tabs-nav__prefix`},m)),W?l(ie,{onResize:this.handleSegmentResize},{default:()=>l("div",{class:`${e}-tabs-rail`,ref:"tabsElRef"},l("div",{class:`${e}-tabs-capsule`,ref:"segmentCapsuleElRef"},l("div",{class:`${e}-tabs-wrapper`},l("div",{class:`${e}-tabs-tab`}))),z?w.map((m,b)=>(v.value.push(m.props.name),l(pe,Object.assign({},m.props,{internalCreatedByPane:!0,internalLeftPadded:b!==0}),m.children?{default:m.children.tab}:void 0))):S.map((m,b)=>(v.value.push(m.props.name),b===0?m:Pe(m))))}):l(ie,{onResize:this.handleNavResize},{default:()=>l("div",{class:`${e}-tabs-nav-scroll-wrapper`,ref:"scrollWrapperElRef"},["top","bottom"].includes(j)?l(qt,{ref:"xScrollInstRef",onScroll:this.handleScroll},{default:I}):l("div",{class:`${e}-tabs-nav-y-scroll`,onScroll:this.handleScroll,ref:"yScrollElRef"},I()))}),c&&d&&_?$e(d,!0):null,we($,m=>m&&l("div",{class:`${e}-tabs-nav__suffix`},m))),z&&(this.animated&&(j==="top"||j==="bottom")?l("div",{ref:"tabsPaneWrapperRef",style:g,class:[`${e}-tabs-pane-wrapper`,p]},Te(w,this.mergedValue,this.renderedNames,this.onAnimationBeforeLeave,this.onAnimationEnter,this.onAnimationAfterEnter,this.animationDirection)):Te(w,this.mergedValue,this.renderedNames)))}});function Te(e,r,i,c,d,h,v){const u=[];return e.forEach(p=>{const{name:g,displayDirective:x,"display-directive":R}=p.props,$=S=>x===S||R===S,w=r===g;if(p.key!==void 0&&(p.key=g),w||$("show")||$("show:lazy")&&i.has(g)){i.has(g)||i.add(g);const S=!$("if");u.push(S?Ht(p,[[Mt,w]]):p)}}),v?l(Ft,{name:`${v}-transition`,onBeforeLeave:c,onEnter:d,onAfterEnter:h},{default:()=>u}):u}function $e(e,r){return l(pe,{ref:"addTabInstRef",key:"__addable",name:"__addable",internalCreatedByPane:!0,internalAddable:!0,internalLeftPadded:r,disabled:typeof e=="object"&&e.disabled})}function Pe(e){const r=Dt(e);return r.props?r.props.internalLeftPadded=!0:r.props={internalLeftPadded:!0},r}function be(e){return Array.isArray(e.dynamicProps)?e.dynamicProps.includes("internalLeftPadded")||e.dynamicProps.push("internalLeftPadded"):e.dynamicProps=["internalLeftPadded"],e}export{Ca as _,Ra as a,wa as b};

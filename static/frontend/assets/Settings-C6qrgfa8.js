import{a as Vt,f as Mt,u as Ye}from"./settings-CRgSdub9.js";import{A as Xt,I as Kt}from"./ImageSelector-CbGqI4-Q.js";import{f as Gt,u as Jt,c as qt,d as Yt}from"./changelog-Br1AlDIL.js";import{r as Zt,u as Qt}from"./admin-CmDJs8O6.js";import{ak as re,aM as g,bO as W,cw as ea,bf as ta,ba as $e,c2 as aa,ac as rt,bX as Ze,B as De,bZ as Ve,d as lt,W as na,co as Re,aX as Ke,cz as Ge,cf as G,a5 as J,bi as ia,D as o,H as N,A as B,bh as oa,bs as dt,cy as ve,bE as sa,P as be,bJ as ct,ay as ra,J as w,e as la,l as da,c5 as ca,cI as ut,av as ua,bK as fa,ad as Z,c7 as pa,bk as ba,b as Pe,bP as va,N as ga,O as ma,aB as Oe,V as je,cH as Le,bz as ft,am as ha,cK as xa,m as _a,Z as ya,c6 as wa,bo as Ee,aI as Se,cF as ka,aa as ae,a7 as T,ah as u,cJ as h,bM as ze,bB as A,p as Ca,a9 as oe,af as V,a8 as ne,c8 as K,bQ as Ae,bp as Sa}from"./index-CB_FuwfX.js";import{l as za,h as Qe,b as Pa,r as $a,v as Me,q as Ra,f as Ta,_ as Ia}from"./Tooltip-B7R2rhlV.js";import{u as et,a as Ba,_ as Wa}from"./Input-ap0DxGSW.js";import{u as Oa}from"./use-message-BCRiHBGD.js";import{a as ja,_ as La}from"./FormItem-CMLpXSh_.js";import{_ as Ea}from"./Switch-DfEey712.js";import{_ as Aa}from"./InputNumber-CHOIRpDN.js";import{_ as Na}from"./Space-B7sLKVr9.js";import{_ as Ha}from"./_plugin-vue_export-helper-DlAUqK2U.js";import"./files-Cgy293dq.js";const Ua=Qe(".v-x-scroll",{overflow:"auto",scrollbarWidth:"none"},[Qe("&::-webkit-scrollbar",{width:0,height:0})]),Fa=re({name:"XScroll",props:{disabled:Boolean,onScroll:Function},setup(){const e=W(null);function a(l){!(l.currentTarget.offsetWidth<l.currentTarget.scrollWidth)||l.deltaY===0||(l.currentTarget.scrollLeft+=l.deltaY+l.deltaX,l.preventDefault())}const s=ea();return Ua.mount({id:"vueuc/x-scroll",head:!0,anchorMetaName:za,ssr:s}),Object.assign({selfRef:e,handleWheel:a},{scrollTo(...l){var b;(b=e.value)===null||b===void 0||b.scrollTo(...l)}})},render(){return g("div",{ref:"selfRef",onScroll:this.onScroll,onWheel:this.disabled?void 0:this.handleWheel,class:"v-x-scroll"},this.$slots)}});var Da=/\s/;function Va(e){for(var a=e.length;a--&&Da.test(e.charAt(a)););return a}var Ma=/^\s+/;function Xa(e){return e&&e.slice(0,Va(e)+1).replace(Ma,"")}var tt=NaN,Ka=/^[-+]0x[0-9a-f]+$/i,Ga=/^0b[01]+$/i,Ja=/^0o[0-7]+$/i,qa=parseInt;function at(e){if(typeof e=="number")return e;if(ta(e))return tt;if($e(e)){var a=typeof e.valueOf=="function"?e.valueOf():e;e=$e(a)?a+"":a}if(typeof e!="string")return e===0?e:+e;e=Xa(e);var s=Ga.test(e);return s||Ja.test(e)?qa(e.slice(2),s?2:8):Ka.test(e)?tt:+e}var Ne=function(){return aa.Date.now()},Ya="Expected a function",Za=Math.max,Qa=Math.min;function en(e,a,s){var r,l,b,S,p,c,k=0,m=!1,P=!1,I=!0;if(typeof e!="function")throw new TypeError(Ya);a=at(a)||0,$e(s)&&(m=!!s.leading,P="maxWait"in s,b=P?Za(at(s.maxWait)||0,a):b,I="trailing"in s?!!s.trailing:I);function O(C){var F=r,Q=l;return r=l=void 0,k=C,S=e.apply(Q,F),S}function j(C){return k=C,p=setTimeout(U,a),m?O(C):S}function L(C){var F=C-c,Q=C-k,ee=a-F;return P?Qa(ee,b-Q):ee}function H(C){var F=C-c,Q=C-k;return c===void 0||F>=a||F<0||P&&Q>=b}function U(){var C=Ne();if(H(C))return d(C);p=setTimeout(U,L(C))}function d(C){return p=void 0,I&&r?O(C):(r=l=void 0,S)}function x(){p!==void 0&&clearTimeout(p),k=0,r=c=l=p=void 0}function M(){return p===void 0?S:d(Ne())}function $(){var C=Ne(),F=H(C);if(r=arguments,l=this,c=C,F){if(p===void 0)return j(c);if(P)return clearTimeout(p),p=setTimeout(U,a),O(c)}return p===void 0&&(p=setTimeout(U,a)),S}return $.cancel=x,$.flush=M,$}var tn="Expected a function";function an(e,a,s){var r=!0,l=!0;if(typeof e!="function")throw new TypeError(tn);return $e(s)&&(r="leading"in s?!!s.leading:r,l="trailing"in s?!!s.trailing:l),en(e,a,{leading:r,maxWait:a,trailing:l})}const pt=rt("n-popconfirm"),bt={positiveText:String,negativeText:String,showIcon:{type:Boolean,default:!0},onPositiveClick:{type:Function,required:!0},onNegativeClick:{type:Function,required:!0}},nt=ia(bt),nn=re({name:"NPopconfirmPanel",props:bt,setup(e){const{localeRef:a}=et("Popconfirm"),{inlineThemeDisabled:s}=Re(),{mergedClsPrefixRef:r,mergedThemeRef:l,props:b}=Ke(pt),S=J(()=>{const{common:{cubicBezierEaseInOut:c},self:{fontSize:k,iconSize:m,iconColor:P}}=l.value;return{"--n-bezier":c,"--n-font-size":k,"--n-icon-size":m,"--n-icon-color":P}}),p=s?Ge("popconfirm-panel",void 0,S,b):void 0;return Object.assign(Object.assign({},et("Popconfirm")),{mergedClsPrefix:r,cssVars:s?void 0:S,localizedPositiveText:J(()=>e.positiveText||a.value.positiveText),localizedNegativeText:J(()=>e.negativeText||a.value.negativeText),positiveButtonProps:G(b,"positiveButtonProps"),negativeButtonProps:G(b,"negativeButtonProps"),handlePositiveClick(c){e.onPositiveClick(c)},handleNegativeClick(c){e.onNegativeClick(c)},themeClass:p==null?void 0:p.themeClass,onRender:p==null?void 0:p.onRender})},render(){var e;const{mergedClsPrefix:a,showIcon:s,$slots:r}=this,l=Ze(r.action,()=>this.negativeText===null&&this.positiveText===null?[]:[this.negativeText!==null&&g(De,Object.assign({size:"small",onClick:this.handleNegativeClick},this.negativeButtonProps),{default:()=>this.localizedNegativeText}),this.positiveText!==null&&g(De,Object.assign({size:"small",type:"primary",onClick:this.handlePositiveClick},this.positiveButtonProps),{default:()=>this.localizedPositiveText})]);return(e=this.onRender)===null||e===void 0||e.call(this),g("div",{class:[`${a}-popconfirm__panel`,this.themeClass],style:this.cssVars},Ve(r.default,b=>s||b?g("div",{class:`${a}-popconfirm__body`},s?g("div",{class:`${a}-popconfirm__icon`},Ze(r.icon,()=>[g(lt,{clsPrefix:a},{default:()=>g(na,null)})])):null,b):null),l?g("div",{class:[`${a}-popconfirm__action`]},l):null)}}),on=o("popconfirm",[N("body",`
 font-size: var(--n-font-size);
 display: flex;
 align-items: center;
 flex-wrap: nowrap;
 position: relative;
 `,[N("icon",`
 display: flex;
 font-size: var(--n-icon-size);
 color: var(--n-icon-color);
 transition: color .3s var(--n-bezier);
 margin: 0 8px 0 0;
 `)]),N("action",`
 display: flex;
 justify-content: flex-end;
 `,[B("&:not(:first-child)","margin-top: 8px"),o("button",[B("&:not(:last-child)","margin-right: 8px;")])])]),sn=Object.assign(Object.assign(Object.assign({},ve.props),$a),{positiveText:String,negativeText:String,showIcon:{type:Boolean,default:!0},trigger:{type:String,default:"click"},positiveButtonProps:Object,negativeButtonProps:Object,onPositiveClick:Function,onNegativeClick:Function}),rn=re({name:"Popconfirm",props:sn,slots:Object,__popover__:!0,setup(e){const{mergedClsPrefixRef:a}=Re(),s=ve("Popconfirm","-popconfirm",on,sa,e,a),r=W(null);function l(p){var c;if(!(!((c=r.value)===null||c===void 0)&&c.getMergedShow()))return;const{onPositiveClick:k,"onUpdate:show":m}=e;Promise.resolve(k?k(p):!0).then(P=>{var I;P!==!1&&((I=r.value)===null||I===void 0||I.setShow(!1),m&&be(m,!1))})}function b(p){var c;if(!(!((c=r.value)===null||c===void 0)&&c.getMergedShow()))return;const{onNegativeClick:k,"onUpdate:show":m}=e;Promise.resolve(k?k(p):!0).then(P=>{var I;P!==!1&&((I=r.value)===null||I===void 0||I.setShow(!1),m&&be(m,!1))})}return ct(pt,{mergedThemeRef:s,mergedClsPrefixRef:a,props:e}),{setShow(p){var c;(c=r.value)===null||c===void 0||c.setShow(p)},syncPosition(){var p;(p=r.value)===null||p===void 0||p.syncPosition()},mergedTheme:s,popoverInstRef:r,handlePositiveClick:l,handleNegativeClick:b}},render(){const{$slots:e,$props:a,mergedTheme:s}=this;return g(Pa,Object.assign({},dt(a,nt),{theme:s.peers.Popover,themeOverrides:s.peerOverrides.Popover,internalExtraClass:["popconfirm"],ref:"popoverInstRef"}),{trigger:e.trigger,default:()=>{const r=oa(a,nt);return g(nn,Object.assign({},r,{onPositiveClick:this.handlePositiveClick,onNegativeClick:this.handleNegativeClick}),e)}})}}),ln=B([B("@keyframes spin-rotate",`
 from {
 transform: rotate(0);
 }
 to {
 transform: rotate(360deg);
 }
 `),o("spin-container",`
 position: relative;
 `,[o("spin-body",`
 position: absolute;
 top: 50%;
 left: 50%;
 transform: translateX(-50%) translateY(-50%);
 `,[ra()])]),o("spin-body",`
 display: inline-flex;
 align-items: center;
 justify-content: center;
 flex-direction: column;
 `),o("spin",`
 display: inline-flex;
 height: var(--n-size);
 width: var(--n-size);
 font-size: var(--n-size);
 color: var(--n-color);
 `,[w("rotate",`
 animation: spin-rotate 2s linear infinite;
 `)]),o("spin-description",`
 display: inline-block;
 font-size: var(--n-font-size);
 color: var(--n-text-color);
 transition: color .3s var(--n-bezier);
 margin-top: 8px;
 `),o("spin-content",`
 opacity: 1;
 transition: opacity .3s var(--n-bezier);
 pointer-events: all;
 `,[w("spinning",`
 user-select: none;
 -webkit-user-select: none;
 pointer-events: none;
 opacity: var(--n-opacity-spinning);
 `)])]),dn={small:20,medium:18,large:16},cn=Object.assign(Object.assign(Object.assign({},ve.props),{contentClass:String,contentStyle:[Object,String],description:String,size:{type:[String,Number],default:"medium"},show:{type:Boolean,default:!0},rotate:{type:Boolean,default:!0},spinning:{type:Boolean,validator:()=>!0,default:void 0},delay:Number}),ua),un=re({name:"Spin",props:cn,slots:Object,setup(e){const{mergedClsPrefixRef:a,inlineThemeDisabled:s}=Re(e),r=ve("Spin","-spin",ln,ca,e,a),l=J(()=>{const{size:c}=e,{common:{cubicBezierEaseInOut:k},self:m}=r.value,{opacitySpinning:P,color:I,textColor:O}=m,j=typeof c=="number"?fa(c):m[Z("size",c)];return{"--n-bezier":k,"--n-opacity-spinning":P,"--n-size":j,"--n-color":I,"--n-text-color":O}}),b=s?Ge("spin",J(()=>{const{size:c}=e;return typeof c=="number"?String(c):c[0]}),l,e):void 0,S=Me(e,["spinning","show"]),p=W(!1);return ut(c=>{let k;if(S.value){const{delay:m}=e;if(m){k=window.setTimeout(()=>{p.value=!0},m),c(()=>{clearTimeout(k)});return}}p.value=S.value}),{mergedClsPrefix:a,active:p,mergedStrokeWidth:J(()=>{const{strokeWidth:c}=e;if(c!==void 0)return c;const{size:k}=e;return dn[typeof k=="number"?"medium":k]}),cssVars:s?void 0:l,themeClass:b==null?void 0:b.themeClass,onRender:b==null?void 0:b.onRender}},render(){var e,a;const{$slots:s,mergedClsPrefix:r,description:l}=this,b=s.icon&&this.rotate,S=(l||s.description)&&g("div",{class:`${r}-spin-description`},l||((e=s.description)===null||e===void 0?void 0:e.call(s))),p=s.icon?g("div",{class:[`${r}-spin-body`,this.themeClass]},g("div",{class:[`${r}-spin`,b&&`${r}-spin--rotate`],style:s.default?"":this.cssVars},s.icon()),S):g("div",{class:[`${r}-spin-body`,this.themeClass]},g(la,{clsPrefix:r,style:s.default?"":this.cssVars,stroke:this.stroke,"stroke-width":this.mergedStrokeWidth,radius:this.radius,scale:this.scale,class:`${r}-spin`}),S);return(a=this.onRender)===null||a===void 0||a.call(this),s.default?g("div",{class:[`${r}-spin-container`,this.themeClass],style:this.cssVars},g("div",{class:[`${r}-spin-content`,this.active&&`${r}-spin-content--spinning`,this.contentClass],style:this.contentStyle},s),g(da,{name:"fade-in-transition"},{default:()=>this.active?p:null})):p}}),Je=rt("n-tabs"),vt={tab:[String,Number,Object,Function],name:{type:[String,Number],required:!0},disabled:Boolean,displayDirective:{type:String,default:"if"},closable:{type:Boolean,default:void 0},tabProps:Object,label:[String,Number,Object,Function]},fn=re({__TAB_PANE__:!0,name:"TabPane",alias:["TabPanel"],props:vt,slots:Object,setup(e){const a=Ke(Je,null);return a||pa("tab-pane","`n-tab-pane` must be placed inside `n-tabs`."),{style:a.paneStyleRef,class:a.paneClassRef,mergedClsPrefix:a.mergedClsPrefixRef}},render(){return g("div",{class:[`${this.mergedClsPrefix}-tab-pane`,this.class],style:this.style},this.$slots)}}),pn=Object.assign({internalLeftPadded:Boolean,internalAddable:Boolean,internalCreatedByPane:Boolean},dt(vt,["displayDirective"])),Xe=re({__TAB__:!0,inheritAttrs:!1,name:"Tab",props:pn,setup(e){const{mergedClsPrefixRef:a,valueRef:s,typeRef:r,closableRef:l,tabStyleRef:b,addTabStyleRef:S,tabClassRef:p,addTabClassRef:c,tabChangeIdRef:k,onBeforeLeaveRef:m,triggerRef:P,handleAdd:I,activateTab:O,handleClose:j}=Ke(Je);return{trigger:P,mergedClosable:J(()=>{if(e.internalAddable)return!1;const{closable:L}=e;return L===void 0?l.value:L}),style:b,addStyle:S,tabClass:p,addTabClass:c,clsPrefix:a,value:s,type:r,handleClose(L){L.stopPropagation(),!e.disabled&&j(e.name)},activateTab(){if(e.disabled)return;if(e.internalAddable){I();return}const{name:L}=e,H=++k.id;if(L!==s.value){const{value:U}=m;U?Promise.resolve(U(e.name,s.value)).then(d=>{d&&k.id===H&&O(L)}):O(L)}}}},render(){const{internalAddable:e,clsPrefix:a,name:s,disabled:r,label:l,tab:b,value:S,mergedClosable:p,trigger:c,$slots:{default:k}}=this,m=l??b;return g("div",{class:`${a}-tabs-tab-wrapper`},this.internalLeftPadded?g("div",{class:`${a}-tabs-tab-pad`}):null,g("div",Object.assign({key:s,"data-name":s,"data-disabled":r?!0:void 0},ba({class:[`${a}-tabs-tab`,S===s&&`${a}-tabs-tab--active`,r&&`${a}-tabs-tab--disabled`,p&&`${a}-tabs-tab--closable`,e&&`${a}-tabs-tab--addable`,e?this.addTabClass:this.tabClass],onClick:c==="click"?this.activateTab:void 0,onMouseenter:c==="hover"?this.activateTab:void 0,style:e?this.addStyle:this.style},this.internalCreatedByPane?this.tabProps||{}:this.$attrs)),g("span",{class:`${a}-tabs-tab__label`},e?g(Pe,null,g("div",{class:`${a}-tabs-tab__height-placeholder`}," "),g(lt,{clsPrefix:a},{default:()=>g(Xt,null)})):k?k():typeof m=="object"?m:va(m??s)),p&&this.type==="card"?g(ga,{clsPrefix:a,class:`${a}-tabs-tab__close`,onClick:this.handleClose,disabled:r}):null))}}),bn=o("tabs",`
 box-sizing: border-box;
 width: 100%;
 display: flex;
 flex-direction: column;
 transition:
 background-color .3s var(--n-bezier),
 border-color .3s var(--n-bezier);
`,[w("segment-type",[o("tabs-rail",[B("&.transition-disabled",[o("tabs-capsule",`
 transition: none;
 `)])])]),w("top",[o("tab-pane",`
 padding: var(--n-pane-padding-top) var(--n-pane-padding-right) var(--n-pane-padding-bottom) var(--n-pane-padding-left);
 `)]),w("left",[o("tab-pane",`
 padding: var(--n-pane-padding-right) var(--n-pane-padding-bottom) var(--n-pane-padding-left) var(--n-pane-padding-top);
 `)]),w("left, right",`
 flex-direction: row;
 `,[o("tabs-bar",`
 width: 2px;
 right: 0;
 transition:
 top .2s var(--n-bezier),
 max-height .2s var(--n-bezier),
 background-color .3s var(--n-bezier);
 `),o("tabs-tab",`
 padding: var(--n-tab-padding-vertical); 
 `)]),w("right",`
 flex-direction: row-reverse;
 `,[o("tab-pane",`
 padding: var(--n-pane-padding-left) var(--n-pane-padding-top) var(--n-pane-padding-right) var(--n-pane-padding-bottom);
 `),o("tabs-bar",`
 left: 0;
 `)]),w("bottom",`
 flex-direction: column-reverse;
 justify-content: flex-end;
 `,[o("tab-pane",`
 padding: var(--n-pane-padding-bottom) var(--n-pane-padding-right) var(--n-pane-padding-top) var(--n-pane-padding-left);
 `),o("tabs-bar",`
 top: 0;
 `)]),o("tabs-rail",`
 position: relative;
 padding: 3px;
 border-radius: var(--n-tab-border-radius);
 width: 100%;
 background-color: var(--n-color-segment);
 transition: background-color .3s var(--n-bezier);
 display: flex;
 align-items: center;
 `,[o("tabs-capsule",`
 border-radius: var(--n-tab-border-radius);
 position: absolute;
 pointer-events: none;
 background-color: var(--n-tab-color-segment);
 box-shadow: 0 1px 3px 0 rgba(0, 0, 0, .08);
 transition: transform 0.3s var(--n-bezier);
 `),o("tabs-tab-wrapper",`
 flex-basis: 0;
 flex-grow: 1;
 display: flex;
 align-items: center;
 justify-content: center;
 `,[o("tabs-tab",`
 overflow: hidden;
 border-radius: var(--n-tab-border-radius);
 width: 100%;
 display: flex;
 align-items: center;
 justify-content: center;
 `,[w("active",`
 font-weight: var(--n-font-weight-strong);
 color: var(--n-tab-text-color-active);
 `),B("&:hover",`
 color: var(--n-tab-text-color-hover);
 `)])])]),w("flex",[o("tabs-nav",`
 width: 100%;
 position: relative;
 `,[o("tabs-wrapper",`
 width: 100%;
 `,[o("tabs-tab",`
 margin-right: 0;
 `)])])]),o("tabs-nav",`
 box-sizing: border-box;
 line-height: 1.5;
 display: flex;
 transition: border-color .3s var(--n-bezier);
 `,[N("prefix, suffix",`
 display: flex;
 align-items: center;
 `),N("prefix","padding-right: 16px;"),N("suffix","padding-left: 16px;")]),w("top, bottom",[B(">",[o("tabs-nav",[o("tabs-nav-scroll-wrapper",[B("&::before",`
 top: 0;
 bottom: 0;
 left: 0;
 width: 20px;
 `),B("&::after",`
 top: 0;
 bottom: 0;
 right: 0;
 width: 20px;
 `),w("shadow-start",[B("&::before",`
 box-shadow: inset 10px 0 8px -8px rgba(0, 0, 0, .12);
 `)]),w("shadow-end",[B("&::after",`
 box-shadow: inset -10px 0 8px -8px rgba(0, 0, 0, .12);
 `)])])])])]),w("left, right",[o("tabs-nav-scroll-content",`
 flex-direction: column;
 `),B(">",[o("tabs-nav",[o("tabs-nav-scroll-wrapper",[B("&::before",`
 top: 0;
 left: 0;
 right: 0;
 height: 20px;
 `),B("&::after",`
 bottom: 0;
 left: 0;
 right: 0;
 height: 20px;
 `),w("shadow-start",[B("&::before",`
 box-shadow: inset 0 10px 8px -8px rgba(0, 0, 0, .12);
 `)]),w("shadow-end",[B("&::after",`
 box-shadow: inset 0 -10px 8px -8px rgba(0, 0, 0, .12);
 `)])])])])]),o("tabs-nav-scroll-wrapper",`
 flex: 1;
 position: relative;
 overflow: hidden;
 `,[o("tabs-nav-y-scroll",`
 height: 100%;
 width: 100%;
 overflow-y: auto; 
 scrollbar-width: none;
 `,[B("&::-webkit-scrollbar, &::-webkit-scrollbar-track-piece, &::-webkit-scrollbar-thumb",`
 width: 0;
 height: 0;
 display: none;
 `)]),B("&::before, &::after",`
 transition: box-shadow .3s var(--n-bezier);
 pointer-events: none;
 content: "";
 position: absolute;
 z-index: 1;
 `)]),o("tabs-nav-scroll-content",`
 display: flex;
 position: relative;
 min-width: 100%;
 min-height: 100%;
 width: fit-content;
 box-sizing: border-box;
 `),o("tabs-wrapper",`
 display: inline-flex;
 flex-wrap: nowrap;
 position: relative;
 `),o("tabs-tab-wrapper",`
 display: flex;
 flex-wrap: nowrap;
 flex-shrink: 0;
 flex-grow: 0;
 `),o("tabs-tab",`
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
 `,[w("disabled",{cursor:"not-allowed"}),N("close",`
 margin-left: 6px;
 transition:
 background-color .3s var(--n-bezier),
 color .3s var(--n-bezier);
 `),N("label",`
 display: flex;
 align-items: center;
 z-index: 1;
 `)]),o("tabs-bar",`
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
 `,[B("&.transition-disabled",`
 transition: none;
 `),w("disabled",`
 background-color: var(--n-tab-text-color-disabled)
 `)]),o("tabs-pane-wrapper",`
 position: relative;
 overflow: hidden;
 transition: max-height .2s var(--n-bezier);
 `),o("tab-pane",`
 color: var(--n-pane-text-color);
 width: 100%;
 transition:
 color .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 opacity .2s var(--n-bezier);
 left: 0;
 right: 0;
 top: 0;
 `,[B("&.next-transition-leave-active, &.prev-transition-leave-active, &.next-transition-enter-active, &.prev-transition-enter-active",`
 transition:
 color .3s var(--n-bezier),
 background-color .3s var(--n-bezier),
 transform .2s var(--n-bezier),
 opacity .2s var(--n-bezier);
 `),B("&.next-transition-leave-active, &.prev-transition-leave-active",`
 position: absolute;
 `),B("&.next-transition-enter-from, &.prev-transition-leave-to",`
 transform: translateX(32px);
 opacity: 0;
 `),B("&.next-transition-leave-to, &.prev-transition-enter-from",`
 transform: translateX(-32px);
 opacity: 0;
 `),B("&.next-transition-leave-from, &.next-transition-enter-to, &.prev-transition-leave-from, &.prev-transition-enter-to",`
 transform: translateX(0);
 opacity: 1;
 `)]),o("tabs-tab-pad",`
 box-sizing: border-box;
 width: var(--n-tab-gap);
 flex-grow: 0;
 flex-shrink: 0;
 `),w("line-type, bar-type",[o("tabs-tab",`
 font-weight: var(--n-tab-font-weight);
 box-sizing: border-box;
 vertical-align: bottom;
 `,[B("&:hover",{color:"var(--n-tab-text-color-hover)"}),w("active",`
 color: var(--n-tab-text-color-active);
 font-weight: var(--n-tab-font-weight-active);
 `),w("disabled",{color:"var(--n-tab-text-color-disabled)"})])]),o("tabs-nav",[w("line-type",[w("top",[N("prefix, suffix",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `),o("tabs-nav-scroll-content",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `),o("tabs-bar",`
 bottom: -1px;
 `)]),w("left",[N("prefix, suffix",`
 border-right: 1px solid var(--n-tab-border-color);
 `),o("tabs-nav-scroll-content",`
 border-right: 1px solid var(--n-tab-border-color);
 `),o("tabs-bar",`
 right: -1px;
 `)]),w("right",[N("prefix, suffix",`
 border-left: 1px solid var(--n-tab-border-color);
 `),o("tabs-nav-scroll-content",`
 border-left: 1px solid var(--n-tab-border-color);
 `),o("tabs-bar",`
 left: -1px;
 `)]),w("bottom",[N("prefix, suffix",`
 border-top: 1px solid var(--n-tab-border-color);
 `),o("tabs-nav-scroll-content",`
 border-top: 1px solid var(--n-tab-border-color);
 `),o("tabs-bar",`
 top: -1px;
 `)]),N("prefix, suffix",`
 transition: border-color .3s var(--n-bezier);
 `),o("tabs-nav-scroll-content",`
 transition: border-color .3s var(--n-bezier);
 `),o("tabs-bar",`
 border-radius: 0;
 `)]),w("card-type",[N("prefix, suffix",`
 transition: border-color .3s var(--n-bezier);
 `),o("tabs-pad",`
 flex-grow: 1;
 transition: border-color .3s var(--n-bezier);
 `),o("tabs-tab-pad",`
 transition: border-color .3s var(--n-bezier);
 `),o("tabs-tab",`
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
 `,[w("addable",`
 padding-left: 8px;
 padding-right: 8px;
 font-size: 16px;
 justify-content: center;
 `,[N("height-placeholder",`
 width: 0;
 font-size: var(--n-tab-font-size);
 `),ma("disabled",[B("&:hover",`
 color: var(--n-tab-text-color-hover);
 `)])]),w("closable","padding-right: 8px;"),w("active",`
 background-color: #0000;
 font-weight: var(--n-tab-font-weight-active);
 color: var(--n-tab-text-color-active);
 `),w("disabled","color: var(--n-tab-text-color-disabled);")])]),w("left, right",`
 flex-direction: column; 
 `,[N("prefix, suffix",`
 padding: var(--n-tab-padding-vertical);
 `),o("tabs-wrapper",`
 flex-direction: column;
 `),o("tabs-tab-wrapper",`
 flex-direction: column;
 `,[o("tabs-tab-pad",`
 height: var(--n-tab-gap-vertical);
 width: 100%;
 `)])]),w("top",[w("card-type",[o("tabs-scroll-padding","border-bottom: 1px solid var(--n-tab-border-color);"),N("prefix, suffix",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `),o("tabs-tab",`
 border-top-left-radius: var(--n-tab-border-radius);
 border-top-right-radius: var(--n-tab-border-radius);
 `,[w("active",`
 border-bottom: 1px solid #0000;
 `)]),o("tabs-tab-pad",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `),o("tabs-pad",`
 border-bottom: 1px solid var(--n-tab-border-color);
 `)])]),w("left",[w("card-type",[o("tabs-scroll-padding","border-right: 1px solid var(--n-tab-border-color);"),N("prefix, suffix",`
 border-right: 1px solid var(--n-tab-border-color);
 `),o("tabs-tab",`
 border-top-left-radius: var(--n-tab-border-radius);
 border-bottom-left-radius: var(--n-tab-border-radius);
 `,[w("active",`
 border-right: 1px solid #0000;
 `)]),o("tabs-tab-pad",`
 border-right: 1px solid var(--n-tab-border-color);
 `),o("tabs-pad",`
 border-right: 1px solid var(--n-tab-border-color);
 `)])]),w("right",[w("card-type",[o("tabs-scroll-padding","border-left: 1px solid var(--n-tab-border-color);"),N("prefix, suffix",`
 border-left: 1px solid var(--n-tab-border-color);
 `),o("tabs-tab",`
 border-top-right-radius: var(--n-tab-border-radius);
 border-bottom-right-radius: var(--n-tab-border-radius);
 `,[w("active",`
 border-left: 1px solid #0000;
 `)]),o("tabs-tab-pad",`
 border-left: 1px solid var(--n-tab-border-color);
 `),o("tabs-pad",`
 border-left: 1px solid var(--n-tab-border-color);
 `)])]),w("bottom",[w("card-type",[o("tabs-scroll-padding","border-top: 1px solid var(--n-tab-border-color);"),N("prefix, suffix",`
 border-top: 1px solid var(--n-tab-border-color);
 `),o("tabs-tab",`
 border-bottom-left-radius: var(--n-tab-border-radius);
 border-bottom-right-radius: var(--n-tab-border-radius);
 `,[w("active",`
 border-top: 1px solid #0000;
 `)]),o("tabs-tab-pad",`
 border-top: 1px solid var(--n-tab-border-color);
 `),o("tabs-pad",`
 border-top: 1px solid var(--n-tab-border-color);
 `)])])])]),He=an,vn=Object.assign(Object.assign({},ve.props),{value:[String,Number],defaultValue:[String,Number],trigger:{type:String,default:"click"},type:{type:String,default:"bar"},closable:Boolean,justifyContent:String,size:String,placement:{type:String,default:"top"},tabStyle:[String,Object],tabClass:String,addTabStyle:[String,Object],addTabClass:String,barWidth:Number,paneClass:String,paneStyle:[String,Object],paneWrapperClass:String,paneWrapperStyle:[String,Object],addable:[Boolean,Object],tabsPadding:{type:Number,default:0},animated:Boolean,onBeforeLeave:Function,onAdd:Function,"onUpdate:value":[Function,Array],onUpdateValue:[Function,Array],onClose:[Function,Array],labelSize:String,activeName:[String,Number],onActiveNameChange:[Function,Array]}),gn=re({name:"Tabs",props:vn,slots:Object,setup(e,{slots:a}){var s,r,l,b;const{mergedClsPrefixRef:S,inlineThemeDisabled:p,mergedComponentPropsRef:c}=Re(e),k=ve("Tabs","-tabs",bn,wa,e,S),m=W(null),P=W(null),I=W(null),O=W(null),j=W(null),L=W(null),H=W(!0),U=W(!0),d=Me(e,["labelSize","size"]),x=J(()=>{var n,i;if(d.value)return d.value;const v=(i=(n=c==null?void 0:c.value)===null||n===void 0?void 0:n.Tabs)===null||i===void 0?void 0:i.size;return v||"medium"}),M=Me(e,["activeName","value"]),$=W((r=(s=M.value)!==null&&s!==void 0?s:e.defaultValue)!==null&&r!==void 0?r:a.default?(b=(l=Oe(a.default())[0])===null||l===void 0?void 0:l.props)===null||b===void 0?void 0:b.name:null),C=Ba(M,$),F={id:0},Q=J(()=>{if(!(!e.justifyContent||e.type==="card"))return{display:"flex",justifyContent:e.justifyContent}});Le(C,()=>{F.id=0,le(),we()});function ee(){var n;const{value:i}=C;return i===null?null:(n=m.value)===null||n===void 0?void 0:n.querySelector(`[data-name="${i}"]`)}function _e(n){if(e.type==="card")return;const{value:i}=P;if(!i)return;const v=i.style.opacity==="0";if(n){const z=`${S.value}-tabs-bar--disabled`,{barWidth:E,placement:q}=e;if(n.dataset.disabled==="true"?i.classList.add(z):i.classList.remove(z),["top","bottom"].includes(q)){if(ye(["top","maxHeight","height"]),typeof E=="number"&&n.offsetWidth>=E){const Y=Math.floor((n.offsetWidth-E)/2)+n.offsetLeft;i.style.left=`${Y}px`,i.style.maxWidth=`${E}px`}else i.style.left=`${n.offsetLeft}px`,i.style.maxWidth=`${n.offsetWidth}px`;i.style.width="8192px",v&&(i.style.transition="none"),i.offsetWidth,v&&(i.style.transition="",i.style.opacity="1")}else{if(ye(["left","maxWidth","width"]),typeof E=="number"&&n.offsetHeight>=E){const Y=Math.floor((n.offsetHeight-E)/2)+n.offsetTop;i.style.top=`${Y}px`,i.style.maxHeight=`${E}px`}else i.style.top=`${n.offsetTop}px`,i.style.maxHeight=`${n.offsetHeight}px`;i.style.height="8192px",v&&(i.style.transition="none"),i.offsetHeight,v&&(i.style.transition="",i.style.opacity="1")}}}function Te(){if(e.type==="card")return;const{value:n}=P;n&&(n.style.opacity="0")}function ye(n){const{value:i}=P;if(i)for(const v of n)i.style[v]=""}function le(){if(e.type==="card")return;const n=ee();n?_e(n):Te()}function we(){var n;const i=(n=j.value)===null||n===void 0?void 0:n.$el;if(!i)return;const v=ee();if(!v)return;const{scrollLeft:z,offsetWidth:E}=i,{offsetLeft:q,offsetWidth:Y}=v;z>q?i.scrollTo({top:0,left:q,behavior:"smooth"}):q+Y>z+E&&i.scrollTo({top:0,left:q+Y-E,behavior:"smooth"})}const _=W(null);let t=0,y=null;function R(n){const i=_.value;if(i){t=n.getBoundingClientRect().height;const v=`${t}px`,z=()=>{i.style.height=v,i.style.maxHeight=v};y?(z(),y(),y=null):y=z}}function D(n){const i=_.value;if(i){const v=n.getBoundingClientRect().height,z=()=>{document.body.offsetHeight,i.style.maxHeight=`${v}px`,i.style.height=`${Math.max(t,v)}px`};y?(y(),y=null,z()):y=z}}function de(){const n=_.value;if(n){n.style.maxHeight="",n.style.height="";const{paneWrapperStyle:i}=e;if(typeof i=="string")n.style.cssText=i;else if(i){const{maxHeight:v,height:z}=i;v!==void 0&&(n.style.maxHeight=v),z!==void 0&&(n.style.height=z)}}}const te={value:[]},ge=W("next");function me(n){const i=C.value;let v="next";for(const z of te.value){if(z===i)break;if(z===n){v="prev";break}}ge.value=v,ce(n)}function ce(n){const{onActiveNameChange:i,onUpdateValue:v,"onUpdate:value":z}=e;i&&be(i,n),v&&be(v,n),z&&be(z,n),$.value=n}function he(n){const{onClose:i}=e;i&&be(i,n)}function ue(){const{value:n}=P;if(!n)return;const i="transition-disabled";n.classList.add(i),le(),n.classList.remove(i)}const se=W(null);function ie({transitionDisabled:n}){const i=m.value;if(!i)return;n&&i.classList.add("transition-disabled");const v=ee();v&&se.value&&(se.value.style.width=`${v.offsetWidth}px`,se.value.style.height=`${v.offsetHeight}px`,se.value.style.transform=`translateX(${v.offsetLeft-ha(getComputedStyle(i).paddingLeft)}px)`,n&&se.value.offsetWidth),n&&i.classList.remove("transition-disabled")}Le([C],()=>{e.type==="segment"&&Ee(()=>{ie({transitionDisabled:!1})})}),ft(()=>{e.type==="segment"&&ie({transitionDisabled:!0})});let ke=0;function Ie(n){var i;if(n.contentRect.width===0&&n.contentRect.height===0||ke===n.contentRect.width)return;ke=n.contentRect.width;const{type:v}=e;if((v==="line"||v==="bar")&&ue(),v!=="segment"){const{placement:z}=e;We((z==="top"||z==="bottom"?(i=j.value)===null||i===void 0?void 0:i.$el:L.value)||null)}}const f=He(Ie,64);Le([()=>e.justifyContent,()=>e.size],()=>{Ee(()=>{const{type:n}=e;(n==="line"||n==="bar")&&ue()})});const X=W(!1);function Be(n){var i;const{target:v,contentRect:{width:z,height:E}}=n,q=v.parentElement.parentElement.offsetWidth,Y=v.parentElement.parentElement.offsetHeight,{placement:pe}=e;if(!X.value)pe==="top"||pe==="bottom"?q<z&&(X.value=!0):Y<E&&(X.value=!0);else{const{value:xe}=O;if(!xe)return;pe==="top"||pe==="bottom"?q-z>xe.$el.offsetWidth&&(X.value=!1):Y-E>xe.$el.offsetHeight&&(X.value=!1)}We(((i=j.value)===null||i===void 0?void 0:i.$el)||null)}const gt=He(Be,64);function mt(){const{onAdd:n}=e;n&&n(),Ee(()=>{const i=ee(),{value:v}=j;!i||!v||v.scrollTo({left:i.offsetLeft,top:0,behavior:"smooth"})})}function We(n){if(!n)return;const{placement:i}=e;if(i==="top"||i==="bottom"){const{scrollLeft:v,scrollWidth:z,offsetWidth:E}=n;H.value=v<=0,U.value=v+E>=z}else{const{scrollTop:v,scrollHeight:z,offsetHeight:E}=n;H.value=v<=0,U.value=v+E>=z}}const ht=He(n=>{We(n.target)},64);ct(Je,{triggerRef:G(e,"trigger"),tabStyleRef:G(e,"tabStyle"),tabClassRef:G(e,"tabClass"),addTabStyleRef:G(e,"addTabStyle"),addTabClassRef:G(e,"addTabClass"),paneClassRef:G(e,"paneClass"),paneStyleRef:G(e,"paneStyle"),mergedClsPrefixRef:S,typeRef:G(e,"type"),closableRef:G(e,"closable"),valueRef:C,tabChangeIdRef:F,onBeforeLeaveRef:G(e,"onBeforeLeave"),activateTab:me,handleClose:he,handleAdd:mt}),Ra(()=>{le(),we()}),ut(()=>{const{value:n}=I;if(!n)return;const{value:i}=S,v=`${i}-tabs-nav-scroll-wrapper--shadow-start`,z=`${i}-tabs-nav-scroll-wrapper--shadow-end`;H.value?n.classList.remove(v):n.classList.add(v),U.value?n.classList.remove(z):n.classList.add(z)});const xt={syncBarPosition:()=>{le()}},_t=()=>{ie({transitionDisabled:!0})},qe=J(()=>{const{value:n}=x,{type:i}=e,v={card:"Card",bar:"Bar",line:"Line",segment:"Segment"}[i],z=`${n}${v}`,{self:{barColor:E,closeIconColor:q,closeIconColorHover:Y,closeIconColorPressed:pe,tabColor:xe,tabBorderColor:yt,paneTextColor:wt,tabFontWeight:kt,tabBorderRadius:Ct,tabFontWeightActive:St,colorSegment:zt,fontWeightStrong:Pt,tabColorSegment:$t,closeSize:Rt,closeIconSize:Tt,closeColorHover:It,closeColorPressed:Bt,closeBorderRadius:Wt,[Z("panePadding",n)]:Ce,[Z("tabPadding",z)]:Ot,[Z("tabPaddingVertical",z)]:jt,[Z("tabGap",z)]:Lt,[Z("tabGap",`${z}Vertical`)]:Et,[Z("tabTextColor",i)]:At,[Z("tabTextColorActive",i)]:Nt,[Z("tabTextColorHover",i)]:Ht,[Z("tabTextColorDisabled",i)]:Ut,[Z("tabFontSize",n)]:Ft},common:{cubicBezierEaseInOut:Dt}}=k.value;return{"--n-bezier":Dt,"--n-color-segment":zt,"--n-bar-color":E,"--n-tab-font-size":Ft,"--n-tab-text-color":At,"--n-tab-text-color-active":Nt,"--n-tab-text-color-disabled":Ut,"--n-tab-text-color-hover":Ht,"--n-pane-text-color":wt,"--n-tab-border-color":yt,"--n-tab-border-radius":Ct,"--n-close-size":Rt,"--n-close-icon-size":Tt,"--n-close-color-hover":It,"--n-close-color-pressed":Bt,"--n-close-border-radius":Wt,"--n-close-icon-color":q,"--n-close-icon-color-hover":Y,"--n-close-icon-color-pressed":pe,"--n-tab-color":xe,"--n-tab-font-weight":kt,"--n-tab-font-weight-active":St,"--n-tab-padding":Ot,"--n-tab-padding-vertical":jt,"--n-tab-gap":Lt,"--n-tab-gap-vertical":Et,"--n-pane-padding-left":Se(Ce,"left"),"--n-pane-padding-right":Se(Ce,"right"),"--n-pane-padding-top":Se(Ce,"top"),"--n-pane-padding-bottom":Se(Ce,"bottom"),"--n-font-weight-strong":Pt,"--n-tab-color-segment":$t}}),fe=p?Ge("tabs",J(()=>`${x.value[0]}${e.type[0]}`),qe,e):void 0;return Object.assign({mergedClsPrefix:S,mergedValue:C,renderedNames:new Set,segmentCapsuleElRef:se,tabsPaneWrapperRef:_,tabsElRef:m,barElRef:P,addTabInstRef:O,xScrollInstRef:j,scrollWrapperElRef:I,addTabFixed:X,tabWrapperStyle:Q,handleNavResize:f,mergedSize:x,handleScroll:ht,handleTabsResize:gt,cssVars:p?void 0:qe,themeClass:fe==null?void 0:fe.themeClass,animationDirection:ge,renderNameListRef:te,yScrollElRef:L,handleSegmentResize:_t,onAnimationBeforeLeave:R,onAnimationEnter:D,onAnimationAfterEnter:de,onRender:fe==null?void 0:fe.onRender},xt)},render(){const{mergedClsPrefix:e,type:a,placement:s,addTabFixed:r,addable:l,mergedSize:b,renderNameListRef:S,onRender:p,paneWrapperClass:c,paneWrapperStyle:k,$slots:{default:m,prefix:P,suffix:I}}=this;p==null||p();const O=m?Oe(m()).filter($=>$.type.__TAB_PANE__===!0):[],j=m?Oe(m()).filter($=>$.type.__TAB__===!0):[],L=!j.length,H=a==="card",U=a==="segment",d=!H&&!U&&this.justifyContent;S.value=[];const x=()=>{const $=g("div",{style:this.tabWrapperStyle,class:`${e}-tabs-wrapper`},d?null:g("div",{class:`${e}-tabs-scroll-padding`,style:s==="top"||s==="bottom"?{width:`${this.tabsPadding}px`}:{height:`${this.tabsPadding}px`}}),L?O.map((C,F)=>(S.value.push(C.props.name),Ue(g(Xe,Object.assign({},C.props,{internalCreatedByPane:!0,internalLeftPadded:F!==0&&(!d||d==="center"||d==="start"||d==="end")}),C.children?{default:C.children.tab}:void 0)))):j.map((C,F)=>(S.value.push(C.props.name),Ue(F!==0&&!d?st(C):C))),!r&&l&&H?ot(l,(L?O.length:j.length)!==0):null,d?null:g("div",{class:`${e}-tabs-scroll-padding`,style:{width:`${this.tabsPadding}px`}}));return g("div",{ref:"tabsElRef",class:`${e}-tabs-nav-scroll-content`},H&&l?g(je,{onResize:this.handleTabsResize},{default:()=>$}):$,H?g("div",{class:`${e}-tabs-pad`}):null,H?null:g("div",{ref:"barElRef",class:`${e}-tabs-bar`}))},M=U?"top":s;return g("div",{class:[`${e}-tabs`,this.themeClass,`${e}-tabs--${a}-type`,`${e}-tabs--${b}-size`,d&&`${e}-tabs--flex`,`${e}-tabs--${M}`],style:this.cssVars},g("div",{class:[`${e}-tabs-nav--${a}-type`,`${e}-tabs-nav--${M}`,`${e}-tabs-nav`]},Ve(P,$=>$&&g("div",{class:`${e}-tabs-nav__prefix`},$)),U?g(je,{onResize:this.handleSegmentResize},{default:()=>g("div",{class:`${e}-tabs-rail`,ref:"tabsElRef"},g("div",{class:`${e}-tabs-capsule`,ref:"segmentCapsuleElRef"},g("div",{class:`${e}-tabs-wrapper`},g("div",{class:`${e}-tabs-tab`}))),L?O.map(($,C)=>(S.value.push($.props.name),g(Xe,Object.assign({},$.props,{internalCreatedByPane:!0,internalLeftPadded:C!==0}),$.children?{default:$.children.tab}:void 0))):j.map(($,C)=>(S.value.push($.props.name),C===0?$:st($))))}):g(je,{onResize:this.handleNavResize},{default:()=>g("div",{class:`${e}-tabs-nav-scroll-wrapper`,ref:"scrollWrapperElRef"},["top","bottom"].includes(M)?g(Fa,{ref:"xScrollInstRef",onScroll:this.handleScroll},{default:x}):g("div",{class:`${e}-tabs-nav-y-scroll`,onScroll:this.handleScroll,ref:"yScrollElRef"},x()))}),r&&l&&H?ot(l,!0):null,Ve(I,$=>$&&g("div",{class:`${e}-tabs-nav__suffix`},$))),L&&(this.animated&&(M==="top"||M==="bottom")?g("div",{ref:"tabsPaneWrapperRef",style:k,class:[`${e}-tabs-pane-wrapper`,c]},it(O,this.mergedValue,this.renderedNames,this.onAnimationBeforeLeave,this.onAnimationEnter,this.onAnimationAfterEnter,this.animationDirection)):it(O,this.mergedValue,this.renderedNames)))}});function it(e,a,s,r,l,b,S){const p=[];return e.forEach(c=>{const{name:k,displayDirective:m,"display-directive":P}=c.props,I=j=>m===j||P===j,O=a===k;if(c.key!==void 0&&(c.key=k),O||I("show")||I("show:lazy")&&s.has(k)){s.has(k)||s.add(k);const j=!I("if");p.push(j?xa(c,[[ka,O]]):c)}}),S?g(_a,{name:`${S}-transition`,onBeforeLeave:r,onEnter:l,onAfterEnter:b},{default:()=>p}):p}function ot(e,a){return g(Xe,{ref:"addTabInstRef",key:"__addable",name:"__addable",internalCreatedByPane:!0,internalAddable:!0,internalLeftPadded:a,disabled:typeof e=="object"&&e.disabled})}function st(e){const a=ya(e);return a.props?a.props.internalLeftPadded=!0:a.props={internalLeftPadded:!0},a}function Ue(e){return Array.isArray(e.dynamicProps)?e.dynamicProps.includes("internalLeftPadded")||e.dynamicProps.push("internalLeftPadded"):e.dynamicProps=["internalLeftPadded"],e}function Fe(e){const a=e.trim();if(!a)return"请输入IP地址";const s=a.lastIndexOf("/");let r=a,l="";if(s>0&&(r=a.substring(0,s),l=a.substring(s+1)),r.includes(":")){if(!hn(r))return`无效的 IPv6 地址：${r}`;if(l){const b=Number(l);if(!Number.isInteger(b)||b<0||b>128)return"CIDR 前缀必须在 0-128 之间"}}else if(r.includes(".")){if(!mn(r))return`无效的 IPv4 地址：${r}`;if(l){const b=Number(l);if(!Number.isInteger(b)||b<0||b>32)return"CIDR 前缀必须在 0-32 之间"}}else return`无法识别的 IP 格式：${a}`;return""}function mn(e){return/^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/.test(e)}function hn(e){if(e==="::"||e==="::1")return!0;let a=e;if(e.includes("::")){const l=e.split("::");if(l.length>2)return!1;const b=l[0]?l[0].split(":"):[],S=l[1]?l[1].split(":"):[];if(b.length+S.length>=8)return!1;const p=8-b.length-S.length,c=Array(p).fill("0");a=[...b,...c,...S].join(":")}const s=a.split(":");if(s.length!==8)return!1;const r=/^[0-9a-fA-F]{1,4}$/;return s.every(l=>r.test(l))}const xn={class:"flex items-center gap-3"},_n={key:0,class:"w-10 h-10 rounded-lg overflow-hidden flex-shrink-0",style:{background:"var(--color-bg)"}},yn=["src"],wn={class:"flex flex-col gap-1 w-full"},kn={class:"ml-2 text-gray-400 text-sm"},Cn={class:"ml-2 text-gray-400 text-sm"},Sn={class:"ml-2 text-gray-400 text-sm"},zn={class:"flex-1 min-w-0"},Pn={class:"flex items-center gap-2 mb-1 flex-wrap"},$n={class:"text-xs",style:{color:"var(--color-text-muted)"}},Rn={class:"text-sm leading-relaxed",style:{color:"var(--color-text)","white-space":"pre-wrap","word-break":"break-word"}},Tn={class:"flex gap-1.5 flex-shrink-0 ml-3"},In={class:"ml-2 text-sm",style:{color:"var(--color-text-muted)"}},Bn={class:"flex flex-col gap-2 w-full"},Wn={class:"flex gap-2 items-center"},On={class:"text-xs",style:{color:"var(--color-text-muted)","max-width":"160px",overflow:"hidden","text-overflow":"ellipsis","white-space":"nowrap"}},jn=["onClick"],Ln={key:0,class:"text-xs",style:{color:"var(--color-text-muted)"}},En={class:"ml-2 text-sm",style:{color:"var(--color-text-muted)"}},An={class:"flex flex-col gap-2 w-full"},Nn={class:"flex gap-2 items-center"},Hn={class:"text-xs",style:{color:"var(--color-text-muted)","max-width":"160px",overflow:"hidden","text-overflow":"ellipsis","white-space":"nowrap"}},Un=["onClick"],Fn={key:0,class:"text-xs",style:{color:"var(--color-text-muted)"}},Dn=re({__name:"Settings",setup(e){const a=Oa(),s=Vt(),r=W(!1),l=W(!1),b=W(!1);function S(_){_.startsWith("http")&&_.includes("/uploads/")?d.site_logo=_.replace(/https?:\/\/[^/]+\/uploads\//,"/uploads/"):d.site_logo=_,b.value=!1}const p=W([]),c=W(!1),k=W(!1),m=ze({version:"",content:""}),P=W(0);function I(_){return new Date(_).toLocaleDateString("zh-CN",{year:"numeric",month:"long",day:"numeric"})}async function O(){c.value=!0;try{const{data:_}=await Gt();p.value=_.data||[]}finally{c.value=!1}}async function j(){var _,t;if(!m.content.trim()){a.error("请输入更新内容");return}k.value=!0;try{P.value>0?(await Jt(P.value,{version:m.version,content:m.content}),a.success("已更新")):(await qt({version:m.version.trim()||void 0,content:m.content}),a.success(m.version.trim()?"版本已发布":"草稿已保存")),m.version="",m.content="",P.value=0,await O()}catch(y){a.error(((t=(_=y.response)==null?void 0:_.data)==null?void 0:t.error)||(P.value>0?"更新失败":"发布失败"))}finally{k.value=!1}}function L(_){P.value=_.id,m.version=_.version,m.content=_.content}function H(){P.value=0,m.version="",m.content=""}async function U(_){var t,y;try{await Yt(_),a.success("已删除"),await O()}catch(R){a.error(((y=(t=R.response)==null?void 0:t.data)==null?void 0:y.error)||"删除失败")}}const d=ze({site_logo:"",site_title:"",site_subtitle:"",site_description:"",friend_links:"[]",comment_moderation:!1,sidebar_collapse:!1,guestbook_enabled:!0,batch_load_size:5,scroll_load_size:3,site_manager:null}),x=ze({whitelist_enabled:!1,whitelist:[],whitelist_ip_input:"",whitelist_remark_input:"",blacklist_enabled:!1,blacklist:[],blacklist_ip_input:"",blacklist_remark_input:""}),M=W(!1);function $(_){const t=_==="whitelist"?x.whitelist:x.blacklist,y=_==="whitelist"?x.whitelist_ip_input:x.blacklist_ip_input,R=_==="whitelist"?x.whitelist_remark_input:x.blacklist_remark_input,D=y.trim();if(!D)return;const de=Fe(D);if(de){a.warning(de);return}t.some(te=>te.ip===D)||t.push({ip:D,remark:R.trim()}),_==="whitelist"?(x.whitelist_ip_input="",x.whitelist_remark_input=""):(x.blacklist_ip_input="",x.blacklist_remark_input="")}function C(_,t){(_==="whitelist"?x.whitelist:x.blacklist).splice(t,1)}async function F(){var _,t;M.value=!0;try{for(const y of x.whitelist){const R=Fe(y.ip);if(R){a.warning(`白名单：${R}`);return}}for(const y of x.blacklist){const R=Fe(y.ip);if(R){a.warning(`黑名单：${R}`);return}}await Ye({ip_whitelist_enabled:x.whitelist_enabled?"true":"false",ip_whitelist:JSON.stringify(x.whitelist),ip_blacklist_enabled:x.blacklist_enabled?"true":"false",ip_blacklist:JSON.stringify(x.blacklist)}),a.success("IP设置已保存")}catch(y){a.error(((t=(_=y.response)==null?void 0:_.data)==null?void 0:t.error)||"保存失败")}finally{M.value=!1}}const Q=ze(new Map),ee=J(()=>{if(!d.site_logo)return"";if(d.site_logo.startsWith("http"))return d.site_logo;if(d.site_logo.startsWith("nr:")){const _=Number(d.site_logo.slice(3));return Q.get(_)||""}return d.site_logo.startsWith("/")?d.site_logo:`/${d.site_logo}`}),_e=W([]);async function Te(){try{const{data:_}=await Qt({page_size:500});_e.value=(_.data.data||[]).map(t=>({label:`${t.display_name||t.username} (${t.role})`,value:t.id}))}catch{}}async function ye(){r.value=!0;try{const{data:_}=await Mt(),t=_.data.settings;Object.assign(d,{site_logo:t.site_logo||"",site_title:t.site_title||"",site_subtitle:t.site_subtitle||"",site_description:t.site_description||"",friend_links:t.friend_links||"[]",comment_moderation:t.comment_moderation==="true",sidebar_collapse:t.sidebar_collapse==="true",guestbook_enabled:t.guestbook_enabled!=="false",batch_load_size:Number(t.batch_load_size)||5,scroll_load_size:Number(t.scroll_load_size)||3,site_manager:t["site-manager"]?Number(t["site-manager"]):null}),x.whitelist_enabled=t.ip_whitelist_enabled==="true",x.whitelist=le(t.ip_whitelist),x.blacklist_enabled=t.ip_blacklist_enabled==="true",x.blacklist=le(t.ip_blacklist)}finally{r.value=!1}}function le(_){if(!_)return[];try{const t=JSON.parse(_);return Array.isArray(t)?t.map(y=>typeof y=="string"?{ip:y,remark:""}:typeof y=="object"&&y!==null?{ip:String(y.ip||""),remark:String(y.remark||"")}:{ip:"",remark:""}).filter(y=>y.ip):[]}catch{return[]}}async function we(){var _,t;l.value=!0;try{await Ye({site_logo:d.site_logo,site_title:d.site_title,site_subtitle:d.site_subtitle,site_description:d.site_description,friend_links:d.friend_links,comment_moderation:d.comment_moderation?"true":"false",sidebar_collapse:d.sidebar_collapse?"true":"false",guestbook_enabled:d.guestbook_enabled?"true":"false",batch_load_size:String(d.batch_load_size),scroll_load_size:String(d.scroll_load_size),"site-manager":d.site_manager?String(d.site_manager):""}),a.success("设置已保存"),await s.fetchSettings()}catch(y){a.error(((t=(_=y.response)==null?void 0:_.data)==null?void 0:t.error)||"保存失败")}finally{l.value=!1}}return ft(()=>{ye(),Te(),O(),Zt({page_size:500,source_type:"image"}).then(({data:_})=>{(_.data||[]).forEach(t=>{Q.set(t.id,t.url)})}).catch(()=>{})}),(_,t)=>{const y=De,R=La,D=Wa,de=Ta,te=Ea,ge=Aa,me=ja,ce=Ca,he=un,ue=fn,se=Na,ie=Ia,ke=rn,Ie=gn;return A(),ae("div",null,[t[41]||(t[41]=T("h2",{class:"font-bold mb-6",style:{color:"var(--input-color)","font-size":"28px"}},"⚙️ 系统设置",-1)),u(Ie,{type:"line",animated:""},{default:h(()=>[u(ue,{name:"settings",tab:"站点设置"},{default:h(()=>[u(he,{show:r.value},{default:h(()=>[u(ce,{class:"mb-4"},{default:h(()=>[u(me,{model:d,"label-placement":"left","label-width":"100"},{default:h(()=>[u(R,{label:"网站 LOGO"},{default:h(()=>[T("div",xn,[ee.value?(A(),ae("div",_n,[T("img",{src:ee.value,class:"w-full h-full object-contain"},null,8,yn)])):oe("",!0),u(y,{size:"small",onClick:t[0]||(t[0]=f=>b.value=!0)},{default:h(()=>[...t[23]||(t[23]=[V("设置图片",-1)])]),_:1}),d.site_logo?(A(),ne(y,{key:1,size:"small",secondary:"",onClick:t[1]||(t[1]=f=>d.site_logo="")},{default:h(()=>[...t[24]||(t[24]=[V("移除",-1)])]),_:1})):oe("",!0)]),t[25]||(t[25]=T("span",{class:"text-xs mt-1",style:{color:"var(--color-text-muted)"}},"留空则使用站点标题作为 LOGO",-1))]),_:1}),u(R,{label:"站点标题"},{default:h(()=>[u(D,{value:d.site_title,"onUpdate:value":t[2]||(t[2]=f=>d.site_title=f),placeholder:"站点标题"},null,8,["value"])]),_:1}),u(R,{label:"站点副标题"},{default:h(()=>[u(D,{value:d.site_subtitle,"onUpdate:value":t[3]||(t[3]=f=>d.site_subtitle=f),placeholder:"一句话标语，如「记录技术、分享生活」"},null,8,["value"])]),_:1}),u(R,{label:"站点描述"},{default:h(()=>[u(D,{value:d.site_description,"onUpdate:value":t[4]||(t[4]=f=>d.site_description=f),type:"textarea",placeholder:"站点描述",rows:10},null,8,["value"])]),_:1}),u(R,{label:"友情链接"},{default:h(()=>[T("div",wn,[u(D,{value:d.friend_links,"onUpdate:value":t[5]||(t[5]=f=>d.friend_links=f),type:"textarea",rows:5},null,8,["value"]),t[26]||(t[26]=T("span",{class:"text-xs",style:{color:"var(--color-text-muted)"}},"JSON 数组格式，每项包含 name（站点名）和 url（链接地址）",-1))])]),_:1}),u(R,{label:"指定站长"},{default:h(()=>[u(de,{value:d.site_manager,"onUpdate:value":t[6]||(t[6]=f=>d.site_manager=f),options:_e.value,placeholder:"选择前台展示的站长用户（留空=首个管理员）",clearable:"",filterable:"",style:{"max-width":"320px"}},null,8,["value","options"])]),_:1}),u(R,{label:"访客留言审核"},{default:h(()=>[u(te,{value:d.comment_moderation,"onUpdate:value":t[7]||(t[7]=f=>d.comment_moderation=f)},null,8,["value"]),T("span",kn,K(d.comment_moderation?"开启":"关闭"),1)]),_:1}),u(R,{label:"侧栏分类折叠"},{default:h(()=>[u(te,{value:d.sidebar_collapse,"onUpdate:value":t[8]||(t[8]=f=>d.sidebar_collapse=f)},null,8,["value"]),T("span",Cn,K(d.sidebar_collapse?"默认折叠":"默认展开"),1)]),_:1}),u(R,{label:"开启留言板"},{default:h(()=>[u(te,{value:d.guestbook_enabled,"onUpdate:value":t[9]||(t[9]=f=>d.guestbook_enabled=f)},null,8,["value"]),T("span",Sn,K(d.guestbook_enabled?"开启":"关闭"),1)]),_:1}),u(R,{label:"批量装载数量"},{default:h(()=>[u(ge,{value:d.batch_load_size,"onUpdate:value":t[10]||(t[10]=f=>d.batch_load_size=f),min:1,max:20,style:{width:"120px"}},null,8,["value"]),t[27]||(t[27]=T("span",{class:"ml-2 text-gray-400 text-sm"},"首页/分类等首次加载的文章数",-1))]),_:1}),u(R,{label:"滚动装载数量"},{default:h(()=>[u(ge,{value:d.scroll_load_size,"onUpdate:value":t[11]||(t[11]=f=>d.scroll_load_size=f),min:1,max:20,style:{width:"120px"}},null,8,["value"]),t[28]||(t[28]=T("span",{class:"ml-2 text-gray-400 text-sm"},"滚动到底时单次追加文章数",-1))]),_:1}),u(R,null,{default:h(()=>[u(y,{type:"primary",loading:l.value,onClick:we},{default:h(()=>[...t[29]||(t[29]=[V("保存设置",-1)])]),_:1},8,["loading"])]),_:1})]),_:1},8,["model"])]),_:1})]),_:1},8,["show"])]),_:1}),u(ue,{name:"changelog",tab:"版本维护"},{default:h(()=>[u(he,{show:c.value},{default:h(()=>[u(ce,{class:"mb-4"},{default:h(()=>[u(me,{"label-placement":"left","label-width":"100"},{default:h(()=>[u(R,{label:"版本号"},{default:h(()=>[u(se,{align:"center"},{default:h(()=>[u(D,{value:m.version,"onUpdate:value":t[12]||(t[12]=f=>m.version=f),placeholder:"留空保存草稿，填写后前台可见",style:{width:"240px"}},null,8,["value"]),P.value>0?(A(),ne(y,{key:0,type:"primary",loading:k.value,onClick:j},{default:h(()=>[...t[30]||(t[30]=[V("保存修改",-1)])]),_:1},8,["loading"])):(A(),ne(y,{key:1,type:"primary",loading:k.value,onClick:j},{default:h(()=>[...t[31]||(t[31]=[V("保存",-1)])]),_:1},8,["loading"])),P.value>0?(A(),ne(y,{key:2,onClick:H},{default:h(()=>[...t[32]||(t[32]=[V("取消编辑",-1)])]),_:1})):oe("",!0)]),_:1})]),_:1}),u(R,{label:"更新内容"},{default:h(()=>[u(D,{value:m.content,"onUpdate:value":t[13]||(t[13]=f=>m.content=f),type:"textarea",placeholder:"Markdown 格式的更新说明",rows:5},null,8,["value"])]),_:1})]),_:1})]),_:1}),p.value.length>0?(A(),ne(ce,{key:0},{default:h(()=>[t[37]||(t[37]=T("h3",{class:"font-bold mb-4",style:{color:"var(--input-color)","font-size":"16px"}},"历史版本信息：",-1)),(A(!0),ae(Pe,null,Ae(p.value,f=>(A(),ae("div",{key:f.id,class:Sa(["version-entry",f.version?"entry-published":"entry-draft"])},[T("div",zn,[T("div",Pn,[f.version?(A(),ne(ie,{key:0,type:"info",size:"small",bordered:!1},{default:h(()=>[V(K(f.version),1)]),_:2},1024)):oe("",!0),f.version?(A(),ne(ie,{key:1,type:"success",size:"small",bordered:!1},{default:h(()=>[...t[33]||(t[33]=[V("已发布",-1)])]),_:1})):(A(),ne(ie,{key:2,type:"warning",size:"small",bordered:!1},{default:h(()=>[...t[34]||(t[34]=[V("草稿",-1)])]),_:1})),T("span",$n,K(I(f.created_at)),1)]),T("p",Rn,K(f.content),1)]),T("div",Tn,[u(y,{size:"tiny",quaternary:"",onClick:X=>L(f)},{default:h(()=>[...t[35]||(t[35]=[V("编辑",-1)])]),_:1},8,["onClick"]),u(ke,{onPositiveClick:X=>U(f.id)},{trigger:h(()=>[u(y,{size:"tiny",type:"error",quaternary:""},{default:h(()=>[...t[36]||(t[36]=[V("删除",-1)])]),_:1})]),default:h(()=>[V(" 确认删除版本 "+K(f.version||"草稿")+"？ ",1)]),_:2},1032,["onPositiveClick"])])],2))),128))]),_:1})):oe("",!0)]),_:1},8,["show"])]),_:1}),u(ue,{name:"ipaccess",tab:"IP访问设置"},{default:h(()=>[u(he,{show:r.value},{default:h(()=>[u(ce,{class:"mb-4"},{default:h(()=>[u(me,{"label-placement":"left","label-width":"120"},{default:h(()=>[u(R,{label:"启用白名单"},{default:h(()=>[u(te,{value:x.whitelist_enabled,"onUpdate:value":t[14]||(t[14]=f=>x.whitelist_enabled=f)},null,8,["value"]),T("span",In,K(x.whitelist_enabled?"仅白名单IP可通过API Key访问":"关闭"),1)]),_:1}),x.whitelist_enabled?(A(),ne(R,{key:0,label:"白名单IP"},{default:h(()=>[T("div",Bn,[T("div",Wn,[u(D,{value:x.whitelist_ip_input,"onUpdate:value":t[15]||(t[15]=f=>x.whitelist_ip_input=f),placeholder:"IP地址",style:{width:"180px"}},null,8,["value"]),u(D,{value:x.whitelist_remark_input,"onUpdate:value":t[16]||(t[16]=f=>x.whitelist_remark_input=f),placeholder:"备注",style:{width:"160px"}},null,8,["value"]),u(y,{size:"small",type:"primary",onClick:t[17]||(t[17]=f=>$("whitelist"))},{default:h(()=>[...t[38]||(t[38]=[V("添加",-1)])]),_:1})]),(A(!0),ae(Pe,null,Ae(x.whitelist,(f,X)=>(A(),ae("div",{key:X,class:"flex items-center gap-2 py-1 px-2 rounded",style:{background:"var(--color-fill-2)"}},[u(ie,{size:"small",type:"success",bordered:!1},{default:h(()=>[V(K(f.ip),1)]),_:2},1024),T("span",On,K(f.remark||"—"),1),T("span",{class:"cursor-pointer",style:{color:"#e74c3c","font-size":"14px","line-height":"1","flex-shrink":"0"},onClick:Be=>C("whitelist",X),title:"删除"},"✕",8,jn)]))),128)),x.whitelist.length?oe("",!0):(A(),ae("span",Ln,"白名单为空时拒绝所有API Key请求"))])]),_:1})):oe("",!0),u(R,{label:"启用黑名单"},{default:h(()=>[u(te,{value:x.blacklist_enabled,"onUpdate:value":t[18]||(t[18]=f=>x.blacklist_enabled=f)},null,8,["value"]),T("span",En,K(x.blacklist_enabled?"黑名单IP完全无法访问网站":"关闭"),1)]),_:1}),x.blacklist_enabled?(A(),ne(R,{key:1,label:"黑名单IP"},{default:h(()=>[T("div",An,[T("div",Nn,[u(D,{value:x.blacklist_ip_input,"onUpdate:value":t[19]||(t[19]=f=>x.blacklist_ip_input=f),placeholder:"IP地址",style:{width:"180px"}},null,8,["value"]),u(D,{value:x.blacklist_remark_input,"onUpdate:value":t[20]||(t[20]=f=>x.blacklist_remark_input=f),placeholder:"备注",style:{width:"160px"}},null,8,["value"]),u(y,{size:"small",type:"primary",onClick:t[21]||(t[21]=f=>$("blacklist"))},{default:h(()=>[...t[39]||(t[39]=[V("添加",-1)])]),_:1})]),(A(!0),ae(Pe,null,Ae(x.blacklist,(f,X)=>(A(),ae("div",{key:X,class:"flex items-center gap-2 py-1 px-2 rounded",style:{background:"var(--color-fill-2)"}},[u(ie,{size:"small",type:"error",bordered:!1},{default:h(()=>[V(K(f.ip),1)]),_:2},1024),T("span",Hn,K(f.remark||"—"),1),T("span",{class:"cursor-pointer",style:{color:"#e74c3c","font-size":"14px","line-height":"1","flex-shrink":"0"},onClick:Be=>C("blacklist",X),title:"删除"},"✕",8,Un)]))),128)),x.blacklist.length?oe("",!0):(A(),ae("span",Fn,"黑名单为空时不影响任何访问"))])]),_:1})):oe("",!0),u(R,null,{default:h(()=>[u(y,{type:"primary",loading:M.value,onClick:F},{default:h(()=>[...t[40]||(t[40]=[V("保存IP设置",-1)])]),_:1},8,["loading"])]),_:1})]),_:1})]),_:1})]),_:1},8,["show"])]),_:1})]),_:1}),u(Kt,{visible:b.value,title:"设置网站 LOGO",onClose:t[22]||(t[22]=f=>b.value=!1),onSelect:S},null,8,["visible"])])}}}),ii=Ha(Dn,[["__scopeId","data-v-47f4db33"]]);export{ii as default};

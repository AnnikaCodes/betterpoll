(window.webpackJsonp=window.webpackJsonp||[]).push([[4,2,3],{254:function(t,e,n){"use strict";n.r(e);var r=n(45),component=Object(r.a)({},(function(){var t=this,e=t.$createElement,n=t._self._c||e;return n("nuxt-link",{staticClass:"button is-info",attrs:{to:"/new"}},[n("b-icon",{attrs:{icon:"ballot"}}),t._v(" "),n("strong",[t._v("Create a poll")])],1)}),[],!1,null,null,null);e.default=component.exports},255:function(t,e,n){t.exports=n.p+"img/navbar-logo.6f28513.png"},256:function(t,e,n){"use strict";n.r(e);var r={props:{current:{type:String,required:!0}}},o=n(45),component=Object(o.a)(r,(function(){var t=this,e=t.$createElement,r=t._self._c||e;return r("b-navbar",{scopedSlots:t._u([{key:"brand",fn:function(){return[r("nuxt-link",{staticClass:"navbar-item",attrs:{to:"/"}},[r("img",{attrs:{src:n(255)}})])]},proxy:!0},{key:"start",fn:function(){return[r("b-navbar-item",{attrs:{tag:"nuxt-link",to:"/"}},[r("span",{class:"/"===t.current?"has-text-primary":""},[t._v("\n                Home\n            ")])]),t._v(" "),r("b-navbar-item",{attrs:{tag:"nuxt-link",to:"/new"}},[r("span",{class:"/new"===t.current?"has-text-primary":""},[t._v("\n                Create\n            ")])]),t._v(" "),r("b-navbar-item",{attrs:{tag:"nuxt-link",to:"/faq"}},[r("span",{class:"/faq"===t.current?"has-text-primary":""},[t._v("\n                Frequently Asked Questions\n            ")])])]},proxy:!0},{key:"end",fn:function(){return[r("b-navbar-item",{staticClass:"buttons"},[r("PollCreateButton")],1)]},proxy:!0}])})}),[],!1,null,null,null);e.default=component.exports;installComponents(component,{PollCreateButton:n(254).default})},259:function(t,e,n){"use strict";n.r(e);var r=n(2).a.extend({name:"AboutPage",head:function(){return{title:"About | BetterPoll"}}}),o=n(45),component=Object(o.a)(r,(function(){var t=this,e=t.$createElement,n=t._self._c||e;return n("main",[n("NavigationMenu",{attrs:{current:"/faq"}}),t._v(" "),t._m(0)],1)}),[function(){var t=this,e=t.$createElement,n=t._self._c||e;return n("section",{staticClass:"hero"},[n("div",{staticClass:"hero-body"},[n("h1",{staticClass:"title"},[t._v("\n              Frequently Asked Questions\n            ")]),t._v(" "),n("h2",{staticClass:"title-small title"},[t._v("\n              What is BetterPoll?\n            ")]),t._v(" "),n("p",[t._v("\n              BetterPoll is a website that lets you quickly and easily\n              make ranked-choice polls and share them on the Internet.\n            ")]),t._v(" "),n("br"),t._v(" "),n("h2",{staticClass:"title-small title"},[t._v("\n              Can I suggest features or report concerns?\n            ")]),t._v(" "),n("p",[t._v("\n                Absolutely! We're always happy to hear feedback; feel free to email us at\n                "),n("a",{attrs:{href:"mailto:support@betterpoll.cc"}},[t._v("support@betterpoll.cc")]),t._v(".\n                "),n("br"),t._v("\n                If you've encountered a bug, you can either send email to the address above, or report it on\n                "),n("a",{attrs:{href:"https://github.com/AnnikaCodes/betterpoll/issues/new"}},[t._v("GitHub")]),t._v(".\n            ")]),t._v(" "),n("br"),t._v(" "),n("h2",{staticClass:"title-small title"},[t._v("\n              I'm a programmer. Can I contribute to BetterPoll or read the source code?\n            ")]),t._v(" "),n("p",[t._v("\n              Yes! BetterPoll is open source; you can find the source code in\n              "),n("a",{attrs:{href:"https://github.com/AnnikaCodes/betterpoll/"}},[t._v("our GitHub repository")]),t._v(".\n              "),n("br"),t._v("\n              The backend is written in Rust and uses PostgreSQL as a database. It uses the\n              "),n("a",{attrs:{href:"https://rocket.rs"}},[t._v("Rocket")]),t._v(" web server and the\n              "),n("code",[n("a",{attrs:{href:"https://docs.rs/tallystick"}},[t._v("tallystick")])]),t._v(" crate to handle vote tallying.\n\n              The frontend is written with the "),n("a",{attrs:{href:"https://vuejs.org"}},[t._v("Vue")]),t._v(" framework\n              and hosted via Nuxt and GitHub Pages.\n            ")]),t._v(" "),n("br"),t._v(" "),n("h2",{staticClass:"title-small title"},[t._v("\n              Who created BetterPoll?\n            ")]),t._v(" "),n("p",[t._v("\n              BetterPoll was created by Annika\n              ("),n("a",{attrs:{href:"https://github.com/AnnikaCodes"}},[t._v("@AnnikaCodes")]),t._v(" on GitHub).\n              Her last name and email address are not publicly available,\n              but if you have a reason to need them or just want to talk to her about BetterPoll,\n              she can be reached at "),n("a",{attrs:{href:"mailto:support@betterpoll.cc"}},[t._v("support@betterpoll.cc")]),t._v(".\n            ")])])])}],!1,null,null,null);e.default=component.exports;installComponents(component,{NavigationMenu:n(256).default})}}]);
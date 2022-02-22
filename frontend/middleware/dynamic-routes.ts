// Based on mktcode's solution
// see https://github.com/mktcode/dynamic-nuxt-gh-pages/blob/main/middleware/gh-pages-dynamic-routes.js

/** Allows the use of #! to redirect to a Nuxt page */
export default function(context: any) {
  const path = context.route.hash.replace('#!', '')
  if (path.length) {
    context.redirect(path)
  }
}

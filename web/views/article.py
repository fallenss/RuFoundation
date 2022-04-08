from django.http import HttpResponse
from django.template import loader

from web.controllers import articles

from renderer import single_pass_render
from renderer.parser import RenderContext


def index(request, article_name='main'):
    template = loader.get_template('page.html')

    # get article itself
    article = articles.get_article(article_name)
    nav_top_article = articles.get_article('nav:top')
    nav_side_article = articles.get_article('nav:side')

    content = single_pass_render(articles.get_latest_source(article), RenderContext(article, article))
    nav_top = single_pass_render(articles.get_latest_source(nav_top_article), RenderContext(article, nav_top_article))
    nav_side = single_pass_render(articles.get_latest_source(nav_side_article), RenderContext(article, nav_side_article))

    context = {
        'content': content,
        'nav_top': nav_top,
        'nav_side': nav_side
    }

    return HttpResponse(template.render(context, request))

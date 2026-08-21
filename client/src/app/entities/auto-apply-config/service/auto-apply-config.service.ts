import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import { Observable } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IAutoApplyConfig, NewAutoApplyConfig } from '../auto-apply-config.model';

export type PartialUpdateAutoApplyConfig = Partial<IAutoApplyConfig> & Pick<IAutoApplyConfig, 'id'>;

@Injectable()
export class AutoApplyConfigsService {
  readonly autoApplyConfigsParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly autoApplyConfigsResource = httpResource<IAutoApplyConfig[]>(() => {
    const params = this.autoApplyConfigsParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of autoApplyConfig that have been fetched. It is updated when the autoApplyConfigsResource emits a new value.
   * In case of error while fetching the autoApplyConfigs, the signal is set to an empty array.
   */
  readonly autoApplyConfigs = computed(() => (this.autoApplyConfigsResource.hasValue() ? this.autoApplyConfigsResource.value() : []));
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/auto-apply-configs');
}

@Injectable({ providedIn: 'root' })
export class AutoApplyConfigService extends AutoApplyConfigsService {
  protected readonly http = inject(HttpClient);

  create(autoApplyConfig: NewAutoApplyConfig): Observable<IAutoApplyConfig> {
    return this.http.post<IAutoApplyConfig>(this.resourceUrl, autoApplyConfig);
  }

  update(autoApplyConfig: IAutoApplyConfig): Observable<IAutoApplyConfig> {
    return this.http.put<IAutoApplyConfig>(
      `${this.resourceUrl}/${encodeURIComponent(this.getAutoApplyConfigIdentifier(autoApplyConfig))}`,
      autoApplyConfig,
    );
  }

  partialUpdate(autoApplyConfig: PartialUpdateAutoApplyConfig): Observable<IAutoApplyConfig> {
    return this.http.patch<IAutoApplyConfig>(
      `${this.resourceUrl}/${encodeURIComponent(this.getAutoApplyConfigIdentifier(autoApplyConfig))}`,
      autoApplyConfig,
    );
  }

  find(id: number): Observable<IAutoApplyConfig> {
    return this.http.get<IAutoApplyConfig>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  query(req?: any): Observable<HttpResponse<IAutoApplyConfig[]>> {
    const options = createRequestOption(req);
    return this.http.get<IAutoApplyConfig[]>(this.resourceUrl, { params: options, observe: 'response' });
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getAutoApplyConfigIdentifier(autoApplyConfig: Pick<IAutoApplyConfig, 'id'>): number {
    return autoApplyConfig.id;
  }

  compareAutoApplyConfig(o1: Pick<IAutoApplyConfig, 'id'> | null, o2: Pick<IAutoApplyConfig, 'id'> | null): boolean {
    return o1 && o2 ? this.getAutoApplyConfigIdentifier(o1) === this.getAutoApplyConfigIdentifier(o2) : o1 === o2;
  }

  addAutoApplyConfigToCollectionIfMissing<Type extends Pick<IAutoApplyConfig, 'id'>>(
    autoApplyConfigCollection: Type[],
    ...autoApplyConfigsToCheck: (Type | null | undefined)[]
  ): Type[] {
    const autoApplyConfigs: Type[] = autoApplyConfigsToCheck.filter(isPresent);
    if (autoApplyConfigs.length > 0) {
      const autoApplyConfigCollectionIdentifiers = autoApplyConfigCollection.map(autoApplyConfigItem =>
        this.getAutoApplyConfigIdentifier(autoApplyConfigItem),
      );
      const autoApplyConfigsToAdd = autoApplyConfigs.filter(autoApplyConfigItem => {
        const autoApplyConfigIdentifier = this.getAutoApplyConfigIdentifier(autoApplyConfigItem);
        if (autoApplyConfigCollectionIdentifiers.includes(autoApplyConfigIdentifier)) {
          return false;
        }
        autoApplyConfigCollectionIdentifiers.push(autoApplyConfigIdentifier);
        return true;
      });
      return [...autoApplyConfigsToAdd, ...autoApplyConfigCollection];
    }
    return autoApplyConfigCollection;
  }
}

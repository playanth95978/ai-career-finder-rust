import { ChangeDetectionStrategy, Component, OnInit, effect, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Data, ParamMap, Router, RouterLink } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap/modal';
import { TranslatePipe } from '@ngx-translate/core';
import { Subscription, combineLatest, filter, tap } from 'rxjs';

import { DEFAULT_SORT_DATA, ITEM_DELETED_EVENT, SORT } from 'app/config/navigation.constants';
import { DataUtils } from 'app/core/util/data-util.service';
import { Alert } from 'app/shared/alert/alert';
import { AlertError } from 'app/shared/alert/alert-error';
import { TranslateDirective } from 'app/shared/language';
import { SortByDirective, SortDirective, SortService, type SortState, sortStateSignal } from 'app/shared/sort';
import { UserPreferenceDeleteDialog } from '../delete/user-preference-delete-dialog';
import { UserPreferenceService } from '../service/user-preference.service';
import { IUserPreference } from '../user-preference.model';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-user-preference',
  templateUrl: './user-preference.html',
  imports: [
    RouterLink,
    FormsModule,
    FontAwesomeModule,
    AlertError,
    Alert,
    SortDirective,
    SortByDirective,
    TranslateDirective,
    TranslatePipe,
  ],
})
export class UserPreference implements OnInit {
  subscription: Subscription | null = null;
  readonly userPreferences = signal<IUserPreference[]>([]);

  sortState = sortStateSignal({});

  readonly router = inject(Router);
  protected readonly userPreferenceService = inject(UserPreferenceService);
  // eslint-disable-next-line @typescript-eslint/member-ordering
  readonly isLoading = this.userPreferenceService.userPreferencesResource.isLoading;
  protected readonly activatedRoute = inject(ActivatedRoute);
  protected readonly sortService = inject(SortService);
  protected dataUtils = inject(DataUtils);
  protected modalService = inject(NgbModal);

  constructor() {
    effect(() => {
      this.userPreferences.set(this.fillComponentAttributesFromResponseBody([...this.userPreferenceService.userPreferences()]));
    });
  }

  trackId = (item: IUserPreference): number => this.userPreferenceService.getUserPreferenceIdentifier(item);

  ngOnInit(): void {
    this.subscription = combineLatest([this.activatedRoute.queryParamMap, this.activatedRoute.data])
      .pipe(
        tap(([params, data]) => this.fillComponentAttributeFromRoute(params, data)),
        tap(() => {
          if (this.userPreferences().length === 0) {
            this.load();
          }
        }),
      )
      .subscribe();
  }

  byteSize(base64String: string): string {
    return this.dataUtils.byteSize(base64String);
  }

  openFile(base64String: string, contentType: string | null | undefined): void {
    return this.dataUtils.openFile(base64String, contentType);
  }

  delete(userPreference: IUserPreference): void {
    const modalRef = this.modalService.open(UserPreferenceDeleteDialog, { size: 'lg', backdrop: 'static' });
    modalRef.componentInstance.userPreference = userPreference;
    // unsubscribe not needed because closed completes on modal close
    modalRef.closed
      .pipe(
        filter(reason => reason === ITEM_DELETED_EVENT),
        tap(() => this.load()),
      )
      .subscribe();
  }

  load(): void {
    this.queryBackend();
  }

  navigateToWithComponentValues(event: SortState): void {
    this.handleNavigation(event);
  }

  protected fillComponentAttributeFromRoute(params: ParamMap, data: Data): void {
    this.sortState.set(this.sortService.parseSortParam(params.get(SORT) ?? data[DEFAULT_SORT_DATA]));
  }

  protected refineData(data: IUserPreference[]): IUserPreference[] {
    const { predicate, order } = this.sortState();
    return predicate && order ? data.sort(this.sortService.startSort({ predicate, order })) : data;
  }

  protected fillComponentAttributesFromResponseBody(data: IUserPreference[]): IUserPreference[] {
    return this.refineData(data);
  }

  protected queryBackend(): void {
    const queryObject: any = {
      sort: this.sortService.buildSortParam(this.sortState()),
    };
    this.userPreferenceService.userPreferencesParams.set(queryObject);
  }

  protected handleNavigation(sortState: SortState): void {
    const queryParamsObj = {
      sort: this.sortService.buildSortParam(sortState),
    };

    this.router.navigate(['./'], {
      relativeTo: this.activatedRoute,
      queryParams: queryParamsObj,
    });
  }
}
